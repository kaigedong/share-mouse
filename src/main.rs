use rdev::{listen, Event};
use std::sync::mpsc;


// 使用libp2p自动发现节点

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

pub struct Server {
    current_client: Option<u32>,
}

impl Server {
    fn new() -> Self {
        Server {}
    }

    async fn send_event(&self, event: Event) {
        let (tx, rx) = mpsc::channel();
        tx.send(event).await;
    }
}
