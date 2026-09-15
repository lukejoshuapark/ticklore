use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait ViewEvent : DeserializeOwned + Send + Serialize {

}
