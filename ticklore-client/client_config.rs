use std::time::Duration;

const DEFAULT_WINDOW_WIDTH: i32 = 1280;
const DEFAULT_WINDOW_HEIGHT: i32 = 720;
const DEFAULT_WINDOW_TITLE: &str = "ticklore";
const DEFAULT_RECONNECT_DELAY: Duration = Duration::from_secs(3);

#[derive(Clone, Debug)]
pub struct ClientConfig {
    pub window_width: i32,
    pub window_height: i32,
    pub window_title: String,
    pub reconnect_delay: Duration
}

impl Default for ClientConfig {
    fn default() -> Self {
        ClientConfig {
            window_width: DEFAULT_WINDOW_WIDTH,
            window_height: DEFAULT_WINDOW_HEIGHT,
            window_title: DEFAULT_WINDOW_TITLE.to_owned(),
            reconnect_delay: DEFAULT_RECONNECT_DELAY
        }
    }
}
