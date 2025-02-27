
## Connecting Readers and Writers

So how do we make sure that messages read in `client` flow into the relevant `client_writer`?
We should somehow maintain an `peers: HashMap<String, Sender<String>>` map which allows a client to find destination channels.
However, this map would be a bit of shared mutable state, so we'll have to wrap an `RwLock` over it and answer tough questions of what should happen if the client joins at the same moment as it receives a message.

One trick to make reasoning about state simpler comes from the actor model.
We can create a dedicated broker tasks which owns the `peers` map and communicates with other tasks by channels.
By hiding `peers` inside such an "actor" task, we remove the need for mutxes and also make serialization point explicit.
The order of events "Bob sends message to Alice" and "Alice joins" is determined by the order of the corresponding events in the broker's event queue.

```rust
#[derive(Debug)]
enum Event { // 1
    NewPeer {
        name: String,
        stream: Arc<TcpStream>,
    },
    Message {
        from: String,
        to: Vec<String>,
        msg: String,
    },
}

async fn broker(mut events: Receiver<Event>) -> Result<()> {
    let mut peers: HashMap<String, Sender<String>> = HashMap::new(); // 2

    while let Some(event) = events.next().await {
        match event {
            Event::Message { from, to, msg } => {  // 3
                for addr in to {
                    if let Some(peer) = peers.get_mut(&addr) {
                        peer.send(format!("from {}: {}\n", from, msg)).await?
                    }
                }
            }
            Event::NewPeer { name, stream } => {
                match peers.entry(name) {
                    Entry::Occupied(..) => (),
                    Entry::Vacant(entry) => {
                        let (client_sender, client_receiver) = mpsc::unbounded();
                        entry.insert(client_sender); // 4
                        spawn_and_log_error(client_writer(client_receiver, stream)); // 5
                    }
                }
            }
        }
    }
    Ok(())
}
```

1. Broker should handle two types of events: a message or an arrival of a new peer.
2. Internal state of the broker is a `HashMap`.
   Note how we don't need a `Mutex` here and can confidently say, at each iteration of the broker's loop, what is the current set of peers
3. To handle a message, we send it over a channel to each destination
4. To handle new peer, we first register it in the peer's map ...
5. ... and then spawn a dedicated task to actually write the messages to the socket.
