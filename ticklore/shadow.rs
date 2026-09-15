use crate::{ShadowEvent, ShadowInput, View};

pub trait Shadow {
    type ShadowEvent : ShadowEvent;
    type View : View;
    type Renderer<'a>;

    fn update(&mut self, inputs: &[ShadowInput<Self::View>], renderer: Self::Renderer<'_>) -> Vec<Self::ShadowEvent>;
}
