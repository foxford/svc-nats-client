use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub url: String,
    pub creds: String,
}
