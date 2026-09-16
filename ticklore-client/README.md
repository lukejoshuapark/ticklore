# ticklore-client

The client half of [ticklore](https://github.com/lukejoshuapark/ticklore).

`start_client` takes an already-open raylib window, connects to a ticklore
server, and drives a `Shadow` until the window is closed:

```rust
use ticklore_client::{ClientConfig, start_client};

let (rl, thread) = raylib::init().size(1280, 720).title("my game").build();

start_client(shadow, "127.0.0.1:9000", ClientConfig::default(), rl, thread);
```

## How it works

Two threads. The render thread calls `Shadow::update` once per frame with
everything that has arrived since the last one, using the raylib handle and
thread the caller opened the window with. A network thread owns the
connection, applies incoming `ViewEvent`s to the view, and forwards the
shadow's events to the server.

The view reaches the shadow as an `Arc` and is updated copy-on-write: if the
shadow let go of the previous frame's view it is updated in place, and a copy
is made only for as long as the shadow holds on. A shadow that reconciles
predicted state against the server's can keep whatever it needs, and one that
does not pays nothing for the option.

## Reconnecting

The client reconnects on its own, waiting `ClientConfig::reconnect_delay`
between attempts. A `Shadow` sees the whole lifecycle through `ShadowInput`:
`Connecting`, `ConnectionFailed`, `Connected` carrying the snapshot that opened
the session, and `Disconnected`.

Events a shadow emits while there is no connection are discarded rather than
replayed into the next one, since the server never saw the session they
belonged to.

Closing the window shuts the network thread down and joins it.

## Building

raylib is built from source, so a C toolchain, CMake, and the usual X11 and GL
development headers must be present. On Debian and Ubuntu:

```
sudo apt install build-essential cmake libasound2-dev libgl1-mesa-dev \
  libglu1-mesa-dev libwayland-dev libx11-dev libxcursor-dev libxi-dev \
  libxinerama-dev libxkbcommon-dev libxrandr-dev
```
