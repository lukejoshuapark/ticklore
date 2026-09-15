use crate::{ClientConfig, drive_connection, drive_shadow};
use ticklore::{Shadow, ShadowInput};

use raylib::drawing::RaylibDrawHandle;

use std::net::ToSocketAddrs;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub fn start_client<S>(
    mut shadow: S,
    server_addr: impl ToSocketAddrs + Send + 'static,
    config: ClientConfig
) where S : for<'a> Shadow<Renderer<'a> = RaylibDrawHandle<'a>> {
    let (outgoing_shadow_inputs, incoming_shadow_inputs) = mpsc::channel::<ShadowInput<S::View>>();
    let (outgoing_shadow_events, incoming_shadow_events) = mpsc::channel::<S::ShadowEvent>();

    let reconnect_delay = config.reconnect_delay;
    let connection_thread = thread::spawn(move || {
        drive_connection::<S>(server_addr, reconnect_delay, outgoing_shadow_inputs, incoming_shadow_events)
    });

    let (mut rl, thread) = raylib::init()
        .size(config.window_width, config.window_height)
        .title(&config.window_title)
        .build();

    let mut shadow_inputs: Vec<ShadowInput<S::View>> = vec![];

    while !rl.window_should_close() {
        let total_time = Duration::from_secs_f64(rl.get_time());
        let delta_time = Duration::from_secs_f32(rl.get_frame_time());

        let d = rl.begin_drawing(&thread);
        drive_shadow(&mut shadow, &mut shadow_inputs, total_time, delta_time, &incoming_shadow_inputs, &outgoing_shadow_events, d);
    }

    drop(outgoing_shadow_events);
    connection_thread.join().ok();
}
