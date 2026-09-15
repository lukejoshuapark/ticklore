use crate::{ShadowEvent, View, WorldEvent, WorldInput};

pub trait World : 'static + Clone + Send + Sync {
    type WorldEvent : WorldEvent;
    type View : View<ViewEvent = <Self::WorldEvent as WorldEvent>::ViewEvent>;
    type ShadowEvent : ShadowEvent;

    fn update(&mut self, inputs: &[WorldInput<Self::ShadowEvent>]) -> Vec<Self::WorldEvent>;
    fn view(&self, client_id: u64) -> Self::View;
}
