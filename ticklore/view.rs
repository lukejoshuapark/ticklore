use crate::ViewEvent;

use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait View : 'static + Clone + DeserializeOwned + Send + Serialize + Sync {
    type ViewEvent : ViewEvent;

    fn apply(&mut self, view_event: &Self::ViewEvent);
}
