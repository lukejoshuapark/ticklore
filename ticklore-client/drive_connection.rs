use crate::{ClientMessage, TransportError, read_transport, write_transport};
use ticklore::{Shadow, ShadowInput, View, ViewEventsShadowInput};

use std::net::{Shutdown, TcpStream, ToSocketAddrs};
use std::ops::ControlFlow;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const DISCONNECT_POLL_INTERVAL: Duration = Duration::from_millis(250);

pub fn drive_connection<S : Shadow>(
    server_addr: impl ToSocketAddrs,
    reconnect_delay: Duration,
    keepalive_interval: Option<Duration>,
    outgoing_shadow_inputs: Sender<ShadowInput<S::View>>,
    incoming_shadow_events: Receiver<S::ShadowEvent>
) {
    loop {
        outgoing_shadow_inputs.send(ShadowInput::Connecting).ok();

        match connect::<S>(&server_addr) {
            Ok((write_stream, read_stream, view)) => {
                let view = Arc::new(view);
                outgoing_shadow_inputs.send(ShadowInput::Connected(Arc::clone(&view))).ok();

                let read_thread = spawn_reader::<S>(read_stream, view, outgoing_shadow_inputs.clone());
                let outcome = run_writer::<S>(write_stream, keepalive_interval, &incoming_shadow_events, &read_thread);
                read_thread.join().ok();

                outgoing_shadow_inputs.send(ShadowInput::Disconnected).ok();

                if outcome.is_break() {
                    return;
                }
            },
            Err(_) => {
                outgoing_shadow_inputs.send(ShadowInput::ConnectionFailed).ok();
            }
        }

        if wait_to_reconnect(&incoming_shadow_events, reconnect_delay).is_break() {
            return;
        }
    }
}

fn connect<S : Shadow>(server_addr: impl ToSocketAddrs) -> Result<(TcpStream, TcpStream, S::View), TransportError> {
    let write_stream = TcpStream::connect(server_addr)?;
    write_stream.set_nodelay(true)?;

    let mut read_stream = write_stream.try_clone()?;
    let view: S::View = read_transport(&mut read_stream)?;

    Ok((write_stream, read_stream, view))
}

fn spawn_reader<S : Shadow>(
    read_stream: TcpStream,
    mut view: Arc<S::View>,
    outgoing_shadow_inputs: Sender<ShadowInput<S::View>>
) -> JoinHandle<()> {
    thread::spawn(move || {
        let read_stream = ShutdownOnDrop(read_stream);

        loop {
            let events: Vec<<S::View as View>::ViewEvent> = match read_transport(&mut &read_stream.0) {
                Ok(events) => events,
                Err(_) => break
            };

            let current_view = Arc::make_mut(&mut view);
            for event in &events {
                current_view.apply(event);
            }

            let view_events_input = ViewEventsShadowInput {
                view: Arc::clone(&view),
                events
            };

            outgoing_shadow_inputs.send(ShadowInput::ViewEvents(view_events_input)).ok();
        }
    })
}

fn run_writer<S : Shadow>(
    mut write_stream: TcpStream,
    keepalive_interval: Option<Duration>,
    incoming_shadow_events: &Receiver<S::ShadowEvent>,
    read_thread: &JoinHandle<()>
) -> ControlFlow<()> {
    while incoming_shadow_events.try_recv().is_ok() { }

    let mut last_write = Instant::now();

    let outcome = loop {
        let poll_interval = match keepalive_interval {
            Some(keepalive_interval) => DISCONNECT_POLL_INTERVAL.min(keepalive_interval.saturating_sub(last_write.elapsed())),
            None => DISCONNECT_POLL_INTERVAL
        };

        let message = match incoming_shadow_events.recv_timeout(poll_interval) {
            Ok(shadow_event) => Some(ClientMessage::ShadowEvent(shadow_event)),
            Err(RecvTimeoutError::Timeout) => keepalive_interval
                .filter(|keepalive_interval| last_write.elapsed() >= *keepalive_interval)
                .map(|_| ClientMessage::Keepalive),
            Err(RecvTimeoutError::Disconnected) => break ControlFlow::Break(())
        };

        if let Some(message) = message {
            if write_transport(&mut write_stream, message).is_err() {
                break ControlFlow::Continue(());
            }

            last_write = Instant::now();
        }

        if read_thread.is_finished() {
            break ControlFlow::Continue(());
        }
    };

    write_stream.shutdown(Shutdown::Both).ok();
    outcome
}

fn wait_to_reconnect<SE>(incoming_shadow_events: &Receiver<SE>, reconnect_delay: Duration) -> ControlFlow<()> {
    let deadline = Instant::now() + reconnect_delay;

    loop {
        let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
            return ControlFlow::Continue(());
        };

        match incoming_shadow_events.recv_timeout(remaining) {
            Ok(_) => { },
            Err(RecvTimeoutError::Timeout) => return ControlFlow::Continue(()),
            Err(RecvTimeoutError::Disconnected) => return ControlFlow::Break(())
        }
    }
}

struct ShutdownOnDrop(TcpStream);

impl Drop for ShutdownOnDrop {
    fn drop(&mut self) {
        self.0.shutdown(Shutdown::Both).ok();
    }
}
