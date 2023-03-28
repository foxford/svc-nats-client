use serde::Deserialize;
use std::time::Duration;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub url: String,
    pub creds: String,
    pub stream: Option<String>,
    pub consumer: Option<String>,
    #[serde(with = "humantime_serde")]
    pub redelivery_interval: Option<Duration>,
}
