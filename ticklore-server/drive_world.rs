use crate::Tick;
use ticklore::{TimeWorldInput, World, WorldInput};

use std::time::{Duration, Instant};

use tokio::select;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{MissedTickBehavior, interval};

const INPUT_BATCH_LIMIT: usize = 1024;

pub async fn drive_world<W : World>(
    mut world: W,
    tick_period: Duration,
    mut incoming_world_inputs: mpsc::UnboundedReceiver<WorldInput<W::ShadowEvent>>,
    outgoing_ticks: broadcast::Sender<Tick<W>>
) {
    let mut world_inputs: Vec<WorldInput<W::ShadowEvent>> = vec![];
    let mut client_connected = false;

    let first_tick = Instant::now();
    let mut last_tick = first_tick;
    let mut ticker = interval(tick_period);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        select! {
            biased;

            _ = ticker.tick() => {
                let now = Instant::now();
                world_inputs.push(WorldInput::Time(TimeWorldInput {
                    total_time: now.duration_since(first_tick),
                    delta_time: now.duration_since(last_tick)
                }));

                last_tick = now;

                let world_events = world.update(&world_inputs);
                let world = client_connected.then(|| world.clone());
                outgoing_ticks.send(Tick::new(world, world_events)).ok();

                world_inputs.clear();
                client_connected = false;
            },
            received = incoming_world_inputs.recv_many(&mut world_inputs, INPUT_BATCH_LIMIT) => {
                if received == 0 {
                    return;
                }

                client_connected |= world_inputs[world_inputs.len() - received..]
                    .iter()
                    .any(|world_input| matches!(world_input, WorldInput::Connection(_)));
            }
        }
    }
}
