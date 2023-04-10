use serde::Deserialize;
use std::time::Duration;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub url: String,
    pub creds: String,
    pub subscribe: Option<SubscribeConfig>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SubscribeConfig {
    pub stream: String,
    pub consumer: String,
    pub batch: usize,
    #[serde(with = "humantime_serde")]
    pub idle_heartbeat: Duration,
}
