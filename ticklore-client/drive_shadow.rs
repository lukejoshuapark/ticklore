use ticklore::{Shadow, ShadowInput, TimeShadowInput};

use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

pub fn drive_shadow<S : Shadow>(
    shadow: &mut S,
    shadow_inputs: &mut Vec<ShadowInput<S::View>>,
    total_time: Duration,
    delta_time: Duration,
    incoming_shadow_inputs: &Receiver<ShadowInput<S::View>>,
    outgoing_shadow_events: &Sender<S::ShadowEvent>,
    renderer: S::Renderer<'_>
) {
    shadow_inputs.clear();
    shadow_inputs.extend(incoming_shadow_inputs.try_iter());
    shadow_inputs.push(ShadowInput::Time(TimeShadowInput { total_time, delta_time }));

    for shadow_event in shadow.update(shadow_inputs, renderer) {
        outgoing_shadow_events.send(shadow_event).ok();
    }
}
