use crate::{Tick, drive_connection};
use ticklore::{World, WorldInput};

use std::io::ErrorKind;
use std::time::Duration;

use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::sync::{broadcast, mpsc};
use tokio::task;
use tokio::time::sleep;
use tracing::{info, warn};

const ACCEPT_BACKOFF_INITIAL: Duration = Duration::from_millis(50);
const ACCEPT_BACKOFF_MAXIMUM: Duration = Duration::from_secs(1);

pub async fn drive_listener<W : World>(
    listen_addr: impl ToSocketAddrs,
    outgoing_world_inputs: mpsc::UnboundedSender<WorldInput<W::ShadowEvent>>,
    outgoing_ticks: broadcast::Sender<Tick<W>>
) -> Result<(), std::io::Error> {
    let listener = TcpListener::bind(listen_addr).await?;

    let (outgoing_cancellation, _) = broadcast::channel::<()>(1);

    info!("Waiting for connections on {}...", listener.local_addr()?);

    let mut accept_backoff = ACCEPT_BACKOFF_INITIAL;

    loop {
        let stream = match listener.accept().await {
            Ok((stream, _)) => {
                accept_backoff = ACCEPT_BACKOFF_INITIAL;
                stream
            },
            Err(err) if is_transient_accept_error(&err) => continue,
            Err(err) => {
                warn!(err = ?err, backoff = ?accept_backoff, "Failed to accept connection, backing off");
                sleep(accept_backoff).await;
                accept_backoff = (accept_backoff * 2).min(ACCEPT_BACKOFF_MAXIMUM);
                continue;
            }
        };

        if let Err(err) = stream.set_nodelay(true) {
            warn!(err = ?err, "Failed to disable Nagle's algorithm, dropping connection");
            continue;
        }

        let outgoing_world_inputs = outgoing_world_inputs.clone();
        let incoming_ticks = outgoing_ticks.subscribe();
        let incoming_cancellation = outgoing_cancellation.subscribe();
        task::spawn(drive_connection::<W>(stream, outgoing_world_inputs, incoming_ticks, incoming_cancellation));
    }
}

fn is_transient_accept_error(err: &std::io::Error) -> bool {
    matches!(
        err.kind(),
        ErrorKind::ConnectionAborted | ErrorKind::ConnectionRefused | ErrorKind::ConnectionReset | ErrorKind::Interrupted
    )
}
