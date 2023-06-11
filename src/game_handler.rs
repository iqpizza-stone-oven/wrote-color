use crate::{ws, Box, Boxes, Result, Clients, Client};
use serde::{Deserialize};
use uuid::Uuid;
use warp::{http::StatusCode, Reply};

#[derive(Deserialize, Debug)]
pub struct BoxCreateRequest {
    color: String
}

pub async fn register_handler(body: BoxCreateRequest, boxes: Boxes) -> Result<impl Reply> {
    let uuid = Uuid::new_v4().as_simple().to_string();
    let color = body.color;
    register_box(uuid.clone(), color, boxes).await;
    Ok(StatusCode::CREATED)
}

async fn register_box(id: String, color: String, boxes: Boxes) {
    boxes.write().await.insert(
        id,
        Box {
            color,
            position: (0, 0)
        }
    );
}

pub async fn ws_handler(ws: warp::ws::Ws, id: String, clients: Clients) -> Result<impl Reply> {
    let mut client = clients.read().await.get(&id).cloned();
    if client.is_none() {
        clients.write().await.insert(id.clone(), Client {
            sender: None,
            word: String::from("None")
        });
        client = clients.read().await.get(&id).cloned();
    }

    println!("{}", client.is_none());
    match client {
        Some(c) => Ok(ws.on_upgrade(move | socket | ws::client_connection(socket, id, clients, c))),
        None => Err(warp::reject::not_found())
    }
}

pub async fn health_handler() -> Result<impl Reply> {
    Ok(StatusCode::OK)
}
