use std::{sync::Arc, time::Duration};

use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use svc_error::extension::sentry;
use tokio::{sync::watch, task::JoinHandle, time::Instant};

use crate::{
    config::ConsumerConfig, AckKind as NatsAckKind, Client, Message, MessageStream, NatsClient,
    SubscribeError,
};

#[derive(Debug)]
pub enum Error {
    SubscriptionFailed(SubscribeError),
    StreamClosed,
    InternalError(anyhow::Error),
    HandleMessageError(anyhow::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::SubscriptionFailed(e) => write!(f, "failed to subscribe to nats: {e}"),
            Error::StreamClosed => write!(f, "nats stream was closed"),
            Error::InternalError(e) => write!(f, "internal nats error: {e}"),
            Error::HandleMessageError(e) => write!(f, "handle message error: {e}"),
        }
    }
}

enum HandleMessageOutcome {
    Processed,
    ProcessLater,
    WontProcess,
}

#[derive(Debug)]
pub enum HandleMessageFailure<E> {
    Transient(E),
    Permanent(E),
}

impl<E> std::fmt::Display for HandleMessageFailure<E>
where
    E: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandleMessageFailure::Transient(e) => write!(f, "transient failure: {e}"),
            HandleMessageFailure::Permanent(e) => write!(f, "permanent failure: {e}"),
        }
    }
}

pub trait FailureKind<T, E> {
    /// This error can be fixed by retrying later.
    fn transient(self) -> Result<T, HandleMessageFailure<E>>;
    /// This error can't be fixed by retrying later (parse failure, unknown id, etc).
    /// Consumer will notify sentry about such errors.
    fn permanent(self) -> Result<T, HandleMessageFailure<E>>;
}

impl<T, E> FailureKind<T, E> for Result<T, E> {
    fn transient(self) -> Result<T, HandleMessageFailure<E>> {
        self.map_err(|e| HandleMessageFailure::Transient(e))
    }

    fn permanent(self) -> Result<T, HandleMessageFailure<E>> {
        self.map_err(|e| HandleMessageFailure::Permanent(e))
    }
}

pub trait FailureKindExt<T, E> {
    /// Maps the internal error E to some other type.
    fn map_err<E1>(self, map: impl FnOnce(E) -> E1) -> Result<T, HandleMessageFailure<E1>>;
}

impl<T, E> FailureKindExt<T, E> for Result<T, HandleMessageFailure<E>> {
    fn map_err<E1>(self, map: impl FnOnce(E) -> E1) -> Result<T, HandleMessageFailure<E1>> {
        self.map_err(|e| match e {
            HandleMessageFailure::Transient(e) => HandleMessageFailure::Transient(map(e)),
            HandleMessageFailure::Permanent(e) => HandleMessageFailure::Permanent(map(e)),
        })
    }
}

pub fn run<H, Fut>(
    nats_client: Client,
    cfg: ConsumerConfig,
    shutdown_rx: watch::Receiver<()>,
    handle_message: H,
) -> JoinHandle<Result<(), SubscribeError>>
where
    H: Fn(Arc<Message>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<(), HandleMessageFailure<anyhow::Error>>>
        + std::marker::Send,
{
    tokio::spawn(async move {
        // In case of subscription errors we don't want to spam sentry
        let mut log_sentry = LogSentry::new(&cfg);

        loop {
            let result = nats_client.subscribe_durable().await;
            let messages = match result {
                Ok(messages) => messages,
                Err(err) => {
                    log_sentry.log_notify(Error::SubscriptionFailed(err));

                    tokio::time::sleep(cfg.resubscribe_interval).await;
                    continue;
                }
            };

            // Run the loop of getting messages from the stream
            let reason = handle_stream(
                &nats_client,
                &cfg,
                messages,
                shutdown_rx.clone(),
                &handle_message,
                &mut log_sentry,
            )
            .await;

            match reason {
                CompletionReason::Shutdown => {
                    tracing::warn!("nats consumer completes its work");
                    break;
                }
                CompletionReason::StreamClosed => {
                    // If the `handle_stream` function ends, then the stream was closed.
                    // Send an error to sentry and try to resubscribe.
                    log_sentry.log_notify(Error::StreamClosed);
                    tokio::time::sleep(cfg.resubscribe_interval).await;
                    continue;
                }
            }
        }

        Ok::<_, SubscribeError>(())
    })
}

enum CompletionReason {
    Shutdown,
    StreamClosed,
}

async fn handle_stream<H, Fut>(
    nats_client: &Client,
    cfg: &ConsumerConfig,
    mut messages: MessageStream,
    mut shutdown_rx: watch::Receiver<()>,
    handle_message: &H,
    log_sentry: &mut LogSentry,
) -> CompletionReason
where
    H: Fn(Arc<Message>) -> Fut,
    Fut: std::future::Future<Output = Result<(), HandleMessageFailure<anyhow::Error>>>,
{
    let mut retry_count = 0;
    let mut suspend_interval: Option<Duration> = None;

    loop {
        if let Some(interval) = suspend_interval.take() {
            tracing::warn!(
                "nats consumer suspenses the processing of nats messages on {} seconds",
                interval.as_secs()
            );
            tokio::time::sleep(interval).await;
        }

        tokio::select! {
            result = messages.next() => {
                let message = match result {
                    Some(Ok(msg)) => msg,
                    Some(Err(err)) => {
                        // Types of internal nats errors that may arise here:
                        // * Heartbeat errors
                        // * Failed to send request
                        // * Consumer deleted
                        // * Received unknown message
                        let err = Error::InternalError(anyhow!(err));
                        log_sentry.log_notify(err);

                        continue;
                    }
                    None => {
                        // Stream was closed. Send an error to sentry and try to resubscribe.
                        return CompletionReason::StreamClosed;
                    }
                };
                let message = Arc::new(message);

                tracing::info!(
                    "got a message from nats, subject: {:?}, payload: {:?}, headers: {:?}",
                    message.subject, message.payload, message.headers
                );

                let outcome = match handle_message(message.clone()).await {
                    Ok(_) => HandleMessageOutcome::Processed,
                    Err(HandleMessageFailure::Transient(e)) => {
                        tracing::error!(%e);
                        HandleMessageOutcome::ProcessLater
                    }
                    Err(HandleMessageFailure::Permanent(e)) => {
                        log_sentry.log_notify(Error::HandleMessageError(e));
                        HandleMessageOutcome::WontProcess
                    }
                };

                match outcome {
                    HandleMessageOutcome::Processed => {
                        retry_count = 0;

                        if let Err(err) = message.ack().await {
                            log_sentry.log_notify(Error::InternalError(anyhow!(err).context("ack failed")));
                        }
                    }
                    HandleMessageOutcome::ProcessLater => {
                        if let Err(err) = message.ack_with(NatsAckKind::Nak(None)).await {
                            log_sentry.log_notify(Error::InternalError(anyhow!(err).context("nack failed")));
                        }

                        retry_count += 1;
                        let interval = next_suspend_interval(retry_count, cfg);
                        suspend_interval = Some(interval);
                    }
                    HandleMessageOutcome::WontProcess => {
                        if let Err(err) = nats_client.terminate(&message).await {
                            log_sentry.log_notify(Error::InternalError(anyhow!(err).context("failed to terminate msg")));
                        }
                    }
                }
            }
            // Graceful shutdown
            _ = shutdown_rx.changed() => {
                return CompletionReason::Shutdown;
            }
        }
    }
}

fn next_suspend_interval(retry_count: u32, nats_consumer_config: &ConsumerConfig) -> Duration {
    let seconds = std::cmp::min(
        nats_consumer_config.suspend_interval.as_secs() * 2_u64.pow(retry_count),
        nats_consumer_config.max_suspend_interval.as_secs(),
    );

    Duration::from_secs(seconds)
}

fn notify_sentry(e: Error) {
    if let Err(e) = sentry::send(Arc::new(anyhow!(e))) {
        tracing::error!("Failed to send error to sentry, reason = {:?}", e);
    }
}

struct LogSentry {
    sentry_last_sent: Instant,
    suspend_interval: Duration,
}

impl LogSentry {
    pub fn new(cfg: &ConsumerConfig) -> Self {
        let sentry_last_sent = Instant::now() - cfg.suspend_sentry_interval * 2;
        Self {
            sentry_last_sent,
            suspend_interval: cfg.suspend_interval,
        }
    }

    pub fn log_notify(&mut self, e: Error) {
        tracing::error!(%e);

        if self.sentry_last_sent.elapsed() >= self.suspend_interval {
            notify_sentry(e);
            self.sentry_last_sent = Instant::now();
        }
    }
}
