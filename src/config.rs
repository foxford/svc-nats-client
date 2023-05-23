use serde::Deserialize;
use std::time::Duration;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub url: String,
    pub creds: String,
    pub subscribe_durable: Option<SubscribeDurableConfig>,
    pub subscribe_ephemeral: Option<SubscribeEphemeralConfig>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SubscribeDurableConfig {
    pub stream: String,
    pub consumer: String,
    pub batch: usize,
    #[serde(with = "humantime_serde")]
    pub idle_heartbeat: Duration,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SubscribeEphemeralConfig {
    pub stream: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConsumerConfig {
    #[serde(with = "humantime_serde")]
    pub suspend_interval: Duration,
    #[serde(with = "humantime_serde")]
    pub max_suspend_interval: Duration,
    #[serde(with = "humantime_serde")]
    pub suspend_sentry_interval: Duration,
    #[serde(with = "humantime_serde")]
    pub resubscribe_interval: Duration,
}
