use crate::ShadowEvent;

use std::time::Duration;

pub enum WorldInput<SE : ShadowEvent> {
    Time(TimeWorldInput),
    Connection(ConnectionWorldInput),
    Disconnection(DisconnectionWorldInput),
    ShadowEvent(ShadowEventWorldInput<SE>)
}

pub struct TimeWorldInput {
    pub total_time: Duration,
    pub delta_time: Duration
}

pub struct ConnectionWorldInput {
    pub client_id: u64
}

pub struct DisconnectionWorldInput {
    pub client_id: u64
}

pub struct ShadowEventWorldInput<SE : ShadowEvent> {
    pub client_id: u64,
    pub shadow_event: SE
}
