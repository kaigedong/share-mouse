use futures_util::StreamExt;
use rdev::Event;
use std::{ops::ControlFlow, sync::mpsc};

// we will use tungstenite for websocket client impl (same library as what axum is using)
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

pub async fn start(tx: mpsc::SyncSender<Event>, connect_to: String) {
    let client = spawn_client(1, tx, connect_to).await;
}

async fn spawn_client(who: usize, tx: mpsc::SyncSender<Event>, connect_to: String) {
    let ws_stream = match connect_async(connect_to + "/ws").await {
        Ok((stream, response)) => {
            println!("Handshake for client {who} has been completed");
            println!("Server response was {response:?}");
            stream
        }
        Err(e) => {
            println!("WebSocket handshake for client {who} failed with {e}!");
            return;
        }
    };

    let (mut _sender, mut receiver) = ws_stream.split();

    //receiver just prints whatever it gets
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            // print message and break if instructed to do so
            if process_message(msg, who, tx.clone()).is_break() {
                break;
            }
        }
    });
}

/// Function to handle messages we get (with a slight twist that Frame variant is visible
/// since we are working with the underlying tungstenite library directly without axum here).
fn process_message(msg: Message, who: usize, tx: mpsc::SyncSender<Event>) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            println!(">>> {who} got str: {t:?}");
        }
        Message::Binary(d) => {
            let event = serde_json::from_slice::<Event>(&d);
            if let Err(e) = &event {
                println!(">>> {who} got binary: {d:?} but failed to parse as Event: {e}");
            }
            let res = tx.try_send(event.unwrap());
            if res.is_err() {
                println!("TrySendEventFailed: {:?}", res);
            }
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                println!(">>> {} got close with code {} and reason `{}`", who, cf.code, cf.reason);
            } else {
                println!(">>> {who} somehow got close message without CloseFrame");
            }
            return ControlFlow::Break(());
        }

        Message::Pong(v) => {
            println!(">>> {who} got pong with {v:?}");
        }
        // Just as with axum server, the underlying tungstenite websocket library
        // will handle Ping for you automagically by replying with Pong and copying the
        // v according to spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            println!(">>> {who} got ping with {v:?}");
        }

        Message::Frame(_) => {
            unreachable!("This is never supposed to happen")
        }
    }
    ControlFlow::Continue(())
}
