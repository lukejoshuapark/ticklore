use ticklore::{World, WorldEvent};

use std::sync::Arc;

pub struct Tick<W : World>(Arc<TickState<W>>);

struct TickState<W : World> {
    world: Option<W>,
    world_events: Vec<W::WorldEvent>
}

impl<W : World> Tick<W> {
    pub fn new(world: Option<W>, world_events: Vec<W::WorldEvent>) -> Self {
        Tick(Arc::new(TickState { world, world_events }))
    }

    pub fn view(&self, client_id: u64) -> Option<W::View> {
        self.0.world.as_ref().map(|x| x.view(client_id))
    }

    pub fn view_events(&self, client_id: u64) -> Vec<<W::WorldEvent as WorldEvent>::ViewEvent> {
        self.0.world_events
            .iter()
            .filter_map(|world_event| world_event.view(client_id))
            .collect()
    }
}

impl<W : World> Clone for Tick<W> {
    fn clone(&self) -> Self {
        Tick(Arc::clone(&self.0))
    }
}
