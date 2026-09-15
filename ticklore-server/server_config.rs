use std::time::Duration;

const DEFAULT_TICK_PERIOD: Duration = Duration::from_nanos(1_000_000_000 / 60);

const DEFAULT_BROADCAST_CAPACITY: usize = 64;

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub tick_period: Duration,
    pub broadcast_capacity: usize
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            tick_period: DEFAULT_TICK_PERIOD,
            broadcast_capacity: DEFAULT_BROADCAST_CAPACITY
        }
    }
}
