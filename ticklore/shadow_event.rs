use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait ShadowEvent : 'static + DeserializeOwned + Send + Serialize {

}
