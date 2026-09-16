use std::time::Duration;

const DEFAULT_RECONNECT_DELAY: Duration = Duration::from_secs(3);

#[derive(Clone, Debug)]
pub struct ClientConfig {
    pub reconnect_delay: Duration
}

impl Default for ClientConfig {
    fn default() -> Self {
        ClientConfig {
            reconnect_delay: DEFAULT_RECONNECT_DELAY
        }
    }
}
