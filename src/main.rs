use anyhow::Result;
use clap::Parser;
use rdev::{listen, Event};
use std::sync::{mpsc, Arc};
use tokio::{signal, sync::Mutex};

mod args;
mod net;
mod traits;

#[tokio::main]
async fn main() -> Result<()> {
    let args = args::Args::parse();
    let server = Arc::new(Server::new(args));
    server.start().await;
    signal::ctrl_c().await?;
    Ok(())
}

pub struct Server {
    args: args::Args,
    s_tx: mpsc::SyncSender<Event>,
    s_rx: Arc<Mutex<mpsc::Receiver<Event>>>,
    c_tx: mpsc::SyncSender<Event>,
    c_rx: Arc<Mutex<mpsc::Receiver<Event>>>,
}

impl Server {
    fn new(args: args::Args) -> Self {
        let (s_tx, s_rx) = mpsc::sync_channel(30);
        let (c_tx, c_rx) = mpsc::sync_channel(30);

        Self {
            args,
            s_tx,
            s_rx: Arc::new(Mutex::new(s_rx)),
            c_tx,
            c_rx: Arc::new(Mutex::new(c_rx)),
        }
    }

    async fn start(self: Arc<Self>) {
        match &self.args.cmd {
            args::Commands::Server { server_listen } => {
                let rx = self.s_rx.clone();
                let server_listen = server_listen.clone();
                tokio::spawn(async move {
                    net::ws_server::start(rx, server_listen).await;
                });
                self.listen_and_send().await;
            }
            args::Commands::Client { connect_to } => {
                let connect_to = connect_to.clone();
                self.recv_signal(connect_to).await
            }
        }
    }

    async fn listen_and_send(self: Arc<Self>) {
        let server = self.clone();
        let tx = Arc::new(server.s_tx.clone());
        tokio::spawn(async move {
            // This will block.
            if let Err(error) = listen(move |event| {
                println!("##### WillSendEvent:{:?}", event);
                if let Err(e) = tx.send(event) {
                    println!("Error: {:?}", e.0);
                };
                println!("##### SendEventFinished");
            }) {
                panic!("Error: {:?}", error)
            }
        });
    }

    async fn recv_signal(self: Arc<Self>, connect_to: String) {
        let c_tx = self.c_tx.clone();
        tokio::spawn(async move {
            net::ws_client::start(c_tx, connect_to).await;
        });

        loop {
            let event = self.c_rx.lock().await.recv().unwrap();
            let res = rdev::simulate(&event.event_type);
            if res.is_err() {
                println!("Error: {:?}", res.err().unwrap());
            }
            // println!("{:?}", event);
        }
    }
}
