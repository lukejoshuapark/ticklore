use crate::{ServerConfig, Tick, drive_listener, drive_world};
use ticklore::{World, WorldInput};

use std::io;

use tokio::net::ToSocketAddrs;
use tokio::select;
use tokio::sync::{broadcast, mpsc};
use tokio::task;

pub async fn start_server<W : World>(world: W, listen_addr: impl ToSocketAddrs, config: ServerConfig) -> io::Result<()> {
    let (outgoing_world_inputs, incoming_world_inputs) = mpsc::unbounded_channel::<WorldInput<W::ShadowEvent>>();
    let (outgoing_ticks, _) = broadcast::channel::<Tick<W>>(config.broadcast_capacity);

    let mut world_task = task::spawn(drive_world(world, config.tick_period, incoming_world_inputs, outgoing_ticks.clone()));

    let result = select! {
        world_result = &mut world_task => match world_result {
            Ok(()) => Err(io::Error::other("the world stopped ticking")),
            Err(err) => Err(io::Error::other(format!("the world panicked: {err}")))
        },
        listener_result = drive_listener(listen_addr, outgoing_world_inputs, outgoing_ticks) => listener_result
    };

    world_task.abort();
    result
}
