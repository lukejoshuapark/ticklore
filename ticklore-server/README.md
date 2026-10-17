# ticklore-server

The server half of [ticklore](https://github.com/lukejoshuapark/ticklore).

`start_server` binds a TCP listener and runs an authoritative `World`:

```rust
use ticklore_server::{ServerConfig, start_server};

start_server(world, "0.0.0.0:9000", ServerConfig::default()).await?;
```

## How it works

One task owns the world and ticks it every `ServerConfig::tick_period`, 60 Hz
by default. Everything that arrives between ticks — connections,
disconnections, and events from clients — is collected and handed to
`World::update` as a single batch. The tick is biased ahead of input handling,
so a flood of client events can slow a tick down but never starve it.

Each tick is broadcast to every connection, which projects it through
`WorldEvent::view` and writes only the events that client is allowed to see. A
tick that produces nothing for a client costs that client nothing.

A joining client is sent a full `View` snapshot before any events. The world is
cloned only on ticks where a client actually joined, so a server with a settled
set of connections never copies its world at all.

## Backpressure

`ServerConfig::broadcast_capacity` bounds how far a connection may fall behind.
A client that lags past it has missed events its view cannot be rebuilt
without, so it is dropped and re-snapshotted when it reconnects. Raising it
buys slow clients more slack at the cost of holding more ticks alive at once.

## Timeouts

`ServerConfig::read_timeout` and `ServerConfig::write_timeout` bound how long a
connection may spend reading or writing a single message before it is dropped.
Both are off by default. The read timeout includes waiting for a message to
begin, so it doubles as an idle timeout: a client that sends nothing for that
long is disconnected, which is how a peer that vanished without closing its
socket is eventually noticed. Clients that may legitimately go quiet should
set `ClientConfig::keepalive_interval` comfortably below the read timeout.

## Failure handling

The listener survives what it can: a peer that gives up before being accepted
is ignored, and errors the process may recover from — descriptor exhaustion
being the common one — back off rather than spin or take the server down. The
world task is supervised, so a world that stops or panics fails the server
instead of leaving the listener accepting clients that will never be ticked.

## Wire format

Each message is a big-endian `u32` payload length followed by that many bytes
of `bincode`. Messages from a client are a `ClientMessage`, either a shadow
event or a keepalive the server discards. Payloads are capped at 4 MiB and the
length is validated before anything is allocated. Both halves of ticklore must
agree on that cap and on the shape of `ClientMessage`.
