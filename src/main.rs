use rdev::{listen, Event};
use std::sync::mpsc;

async fn main() ->Result<()> {
    // This will block.
    if let Err(error) = listen(callback) {
        println!("Error: {:?}", error)
    }

    let server = Server::new();

    server.listen_on("")?;

    loop {
        match server.select_next_some().await {
            ...
        }
    }
}

async fn callback(event: Event) {
    // println!("My callback {:?}", event);
}

pub struct Server {}

impl Server {
    fn new() -> Self {
        Server {}
    }

    async fn send_event(&self, event: Event) {
        let (tx, rx) = mpsc::channel();
        tx.send(event).await;
    }
}
