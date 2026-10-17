use crate::{ClientMessage, HandshakeReply, PROTOCOL_VERSION, ServerConfig, Tick, TransportError, read_handshake, read_transport, with_timeout, write_handshake_reply, write_transport};
use ticklore::{ConnectionWorldInput, DisconnectionWorldInput, ShadowEvent, ShadowEventWorldInput, World, WorldInput};

use std::sync::atomic::{AtomicU64, Ordering};

use tokio::net::TcpStream;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, mpsc};
use tokio::select;
use tracing::{info, warn};

static NEXT_CLIENT_ID: AtomicU64 = AtomicU64::new(0);

pub async fn drive_connection<W : World>(
    mut stream: TcpStream,
    config: ServerConfig,
    outgoing_world_inputs: mpsc::UnboundedSender<WorldInput<W::ShadowEvent>>,
    outgoing_ticks: broadcast::Sender<Tick<W>>,
    mut incoming_cancellation: broadcast::Receiver<()>
) {
    let read_timeout = config.read_timeout;
    let write_timeout = config.write_timeout;

    match handshake(&mut stream, &config).await {
        Ok(HandshakeReply::Accepted) => { },
        Ok(reply) => {
            info!(reply = ?reply, "Client rejected during handshake");
            return;
        },
        Err(err) => {
            warn!(err = ?err, "Failed to complete handshake with client");
            return;
        }
    }

    let mut incoming_ticks = outgoing_ticks.subscribe();

    let client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);
    let connection_world_input = WorldInput::Connection(ConnectionWorldInput { client_id });
    outgoing_world_inputs.send(connection_world_input).ok();

    info!(client_id = client_id, "Client connected");

    let view = loop {
        let tick = match incoming_ticks.recv().await {
            Ok(tick) => tick,
            Err(err) => {
                warn!(client_id = client_id, err = ?err, "Failed to receive first tick, server likely exiting");
                disconnected(client_id, &outgoing_world_inputs);
                return;
            }
        };

        if let Some(view) = tick.view(client_id) {
            break view;
        }
    };

    if let Err(err) = with_timeout(write_timeout, write_transport(&mut stream, view)).await {
        warn!(client_id = client_id, err = ?err, "Failed to send initial view to client, likely disconnected");
        disconnected(client_id, &outgoing_world_inputs);
        return;
    };

    let (mut stream_read, mut stream_write) = stream.into_split();

    let read_future = async {
        loop {
            let shadow_event = match with_timeout(read_timeout, read_transport(&mut stream_read)).await {
                Ok(ClientMessage::ShadowEvent(shadow_event)) => shadow_event,
                Ok(ClientMessage::Keepalive) => continue,
                Err(err) => {
                    warn!(client_id = client_id, err = ?err, "Failed to receive event from client, likely disconnected");
                    break;
                }
            };

            let shadow_event_world_input = WorldInput::ShadowEvent(ShadowEventWorldInput {
                client_id,
                shadow_event
            });

            outgoing_world_inputs.send(shadow_event_world_input).ok();
        }
    };

    let write_future = async {
        loop {
            let tick = match incoming_ticks.recv().await {
                Ok(tick) => tick,
                Err(RecvError::Lagged(missed_ticks)) => {
                    warn!(client_id = client_id, missed_ticks = missed_ticks, "Client fell too far behind to stay in sync, dropping");
                    break;
                },
                Err(RecvError::Closed) => break
            };

            let view_events = tick.view_events(client_id);
            if view_events.is_empty() {
                continue;
            }

            if let Err(err) = with_timeout(write_timeout, write_transport(&mut stream_write, view_events)).await {
                warn!(client_id = client_id, err = ?err, "Failed to send events to client, likely disconnected");
                break;
            };
        }
    };

    select! {
        _ = incoming_cancellation.recv() => { },
        _ = read_future => { },
        _ = write_future => { },
    }

    disconnected(client_id, &outgoing_world_inputs);
}

async fn handshake(stream: &mut TcpStream, config: &ServerConfig) -> Result<HandshakeReply, TransportError> {
    let (protocol_version, version) = with_timeout(config.read_timeout, read_handshake(stream)).await?;

    let reply = if protocol_version != PROTOCOL_VERSION {
        HandshakeReply::ProtocolRejected
    } else if !(config.accept_version)(&version) {
        HandshakeReply::VersionRejected
    } else {
        HandshakeReply::Accepted
    };

    with_timeout(config.write_timeout, write_handshake_reply(stream, reply)).await?;
    Ok(reply)
}

fn disconnected<SE : ShadowEvent>(client_id: u64, outgoing_world_inputs: &mpsc::UnboundedSender<WorldInput<SE>>) {
    let disconnection_world_input = WorldInput::Disconnection(DisconnectionWorldInput { client_id });
    outgoing_world_inputs.send(disconnection_world_input).ok();

    info!(client_id = client_id, "Client disconnected");
}
