use crate::{Client, Clients, Boxes};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{ws::{Message, WebSocket}, Error};

pub async fn client_connection(ws: WebSocket, id: String, clients: Clients, boxes: Boxes, client: Client) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    if !boxes.read().await.is_empty() {
        if let Err(e) = client_sender.send(Ok(Message::text("already"))) {
            eprintln!("{}", e);
            return;
        }
    }
    
    clients.write().await.insert(id.clone(), client);
    let sender = client_sender.clone();
    let stream = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(stream.forward(client_ws_sender).map(move | result: Result<(), Error>| {
        if result.is_err() {
            eprintln!("error sending websocket message");
        }
    }));
    
    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("error receiving ws message for id: {}): {}", id.clone(), e);
                break;
            }
        };
        client_msg(&id, msg, &clients, &boxes, &sender).await;
    }

    clients.write().await.remove(&id);
}

async fn client_msg(id: &str, msg: Message, clients: &Clients, 
            boxes: &Boxes, sender: &UnboundedSender<Result<Message, Error>>) {
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

    if message == "mv_event" || message == "mv_event\n" {
        let mut over_window: Vec<String> = Vec::new();
        for data in boxes.write().await.iter_mut() {
            data.1.position.0 += 24 / clients.read().await.len() as i32;
            let json = Message::text(data.1.to_json());

            if let Err(e) = sender.send(Ok(json)) {
                eprintln!("Error when send boxes move: {}", e);
            }
            
            if data.1.position.0 > 1920 {
                over_window.push(data.0.to_string());
                if let Err(e) = sender.send(Ok(Message::text("over-window"))) {
                    eprintln!("Error when send message to client(overwindow): {}", e);
                    break;
                }
            }
        }

        for entity_key in over_window.iter() {
            boxes.write().await.remove(entity_key);
        }
        return;
    }


    let mut locked = clients.write().await;
    if let Some(v) = locked.get_mut(id) {
        v.word = (&message).to_string();
    }
}
