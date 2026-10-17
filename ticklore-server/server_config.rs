use std::fmt;
use std::sync::Arc;
use std::time::Duration;

const DEFAULT_TICK_PERIOD: Duration = Duration::from_nanos(1_000_000_000 / 60);

const DEFAULT_BROADCAST_CAPACITY: usize = 64;

#[derive(Clone)]
pub struct ServerConfig {
    pub tick_period: Duration,
    pub broadcast_capacity: usize,
    pub read_timeout: Option<Duration>,
    pub write_timeout: Option<Duration>,
    pub accept_version: Arc<dyn Fn(&[u8]) -> bool + Send + Sync>
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            tick_period: DEFAULT_TICK_PERIOD,
            broadcast_capacity: DEFAULT_BROADCAST_CAPACITY,
            read_timeout: None,
            write_timeout: None,
            accept_version: Arc::new(|_| true)
        }
    }
}

impl fmt::Debug for ServerConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServerConfig")
            .field("tick_period", &self.tick_period)
            .field("broadcast_capacity", &self.broadcast_capacity)
            .field("read_timeout", &self.read_timeout)
            .field("write_timeout", &self.write_timeout)
            .finish_non_exhaustive()
    }
}
