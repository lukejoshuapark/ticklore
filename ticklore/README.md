# ticklore

Core traits for [ticklore](https://github.com/lukejoshuapark/ticklore), a TCP
game networking library.

ticklore is server-authoritative. The server owns a `World` and updates it on a
fixed tick, but never sends it to clients. What a client receives is a `View` —
a projection of the world built for that client alone — followed by a stream of
`ViewEvent`s that keep it current.

## The model

`World` is the authoritative state. `World::update` is handed every
`WorldInput` that arrived since the last tick — elapsed time, client
connections and disconnections, and the `ShadowEvent`s clients sent — and
returns the `WorldEvent`s that tick produced. `World::view` projects the world
for one client.

`WorldEvent::view` is the filter. It is called once per connected client, and
returning `None` means that client never learns the event happened, which is
how fog of war and any other per-client secrecy is expressed.

`View` is what a client holds. `View::apply` replays a `ViewEvent` onto it, so
a client's view is the snapshot it joined with plus every event since, in
order.

`Shadow` is the client's presentation layer. It receives `ShadowInput`s —
elapsed time, connection status changes, and each batch of view events
alongside the view they have already been applied to — draws the frame, and
returns `ShadowEvent`s to send back to the server.

`Shadow::Renderer` is an associated type rather than a concrete handle, which
is what keeps rendering out of this crate and so out of the server.
`ticklore-client` binds it to raylib's `RaylibDrawHandle`.

## Notes

- `World::view` can be called with a `client_id` that has not been registered
  by a `ConnectionWorldInput` yet. A client that joins on the same tick as
  another client may be snapshotted a tick before its own connection is
  applied; it learns about itself from the events that follow. Implementations
  should return something sensible rather than panicking.

- Client IDs are unique for the lifetime of a server process, but are not
  stable across restarts and carry no meaning of their own.
