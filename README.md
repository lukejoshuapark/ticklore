# ticklore

A TCP game networking library, split across a few crates:

- [`ticklore`](ticklore) — the core library: the `World`, `View`, and `Shadow` traits and the inputs that drive them.
- [`ticklore-server`](ticklore-server) — runs an authoritative `World` on a fixed tick and serves per-client views over TCP.
- [`ticklore-client`](ticklore-client) — connects to a ticklore server, keeps a `View` up to date, and drives a `Shadow` in a raylib render loop.

See each crate's own README for details.
