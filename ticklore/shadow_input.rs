use crate::View;

use std::sync::Arc;
use std::time::Duration;

pub enum ShadowInput<V : View> {
    Time(TimeShadowInput),
    Connecting,
    ConnectionFailed,
    Connected(Arc<V>),
    Rejected(RejectionReason),
    Disconnected,
    ViewEvents(ViewEventsShadowInput<V>)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RejectionReason {
    Protocol,
    Version
}

pub struct TimeShadowInput {
    pub total_time: Duration,
    pub delta_time: Duration
}

pub struct ViewEventsShadowInput<V : View> {
    pub view: Arc<V>,
    pub events: Vec<V::ViewEvent>
}
