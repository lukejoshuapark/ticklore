use crate::ViewEvent;

pub trait WorldEvent : Send + Sync {
    type ViewEvent : ViewEvent;

    fn view(&self, client_id: u64) -> Option<Self::ViewEvent>;
}
