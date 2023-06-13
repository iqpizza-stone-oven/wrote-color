use crate::{Client, Clients, Boxes};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::time::{self, Duration};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{ws::{Message, WebSocket}, Error};

async fn send_periodic_messages(sender: UnboundedSender<Result<Message, Error>>, boxes: Boxes) {
    let mut interval = time::interval(Duration::from_millis(48));
    let mut over_window: Vec<String> = Vec::with_capacity(10);
    loop {
        interval.tick().await;

        for data in boxes.write().await.iter_mut() {
            data.1.position.0 += 24;
            let json = Message::text(data.1.to_json());

            if let Err(e) = sender.send(Ok(json)) {
                eprintln!("Error when moving boxes: {}", e);
                return;
            }
            if data.1.position.0 >= 1920 {
                over_window.push(data.0.to_string());
            }
        }
        
        for entity_key in over_window.iter() {
            boxes.write().await.remove(entity_key);
        }
    }
}

pub async fn client_connection(ws: WebSocket, id: String, clients: Clients, boxes: Boxes, client: Client) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            eprintln!("error sending websocket msg: {}", e);
        }
    }));
    
    clients.write().await.insert(id.clone(), client);
    let sender = client_sender.clone();

    println!("{} connected", id);
    tokio::task::spawn(send_periodic_messages(client_sender, boxes));
    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("error receiving ws message for id: {}): {}", id.clone(), e);
                break;
            }
        };
        client_msg(&id, msg, &clients, &sender).await;
    }

    clients.write().await.remove(&id);
    println!("{} disconnected", id);
}

async fn client_msg(id: &str, msg: Message, clients: &Clients, sender: &UnboundedSender<Result<Message, Error>>) {
    println!("received message from {}: {:?}", id, msg);
    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };

    if message == "ping" || message == "ping\n" {
        if let Err(e) = sender.send(Ok(Message::text("pong"))) {
            eprintln!("Error when pong: {}", e);
        }
        return;
    }

    let mut locked = clients.write().await;
    if let Some(v) = locked.get_mut(id) {
        v.word = (&message).to_string();
    }
}
