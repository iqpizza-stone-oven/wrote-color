use crate::{ws, Entity, Boxes, Result, Clients, Client};
use serde::{Deserialize};
use uuid::Uuid;
use warp::{http::StatusCode, Reply, reply::with_header};

#[derive(Deserialize, Debug)]
pub struct BoxRequest {
    color: String
}

pub async fn register_handler(body: BoxRequest, boxes: Boxes) -> Result<impl Reply> {
    let uuid = Uuid::new_v4().as_simple().to_string();
    let color = body.color;
    register_box(uuid.clone(), color, boxes).await;
    Ok(with_header(StatusCode::CREATED, "Access-Control-Allow-Origin", "*"))
}

pub async fn answer_handle(body: BoxRequest, boxes: Boxes) -> Result<impl Reply> {
    let result = is_correct(body.color, boxes.clone()).await;
    if result == "null" {
        return Ok(StatusCode:: NOT_FOUND)
    }

    boxes.write().await.remove(&result);
    Ok(StatusCode::OK)
}

async fn is_correct(color: String, boxes: Boxes) -> String {
    for iter in boxes.read().await.clone().into_iter() {
        if iter.1.color == color {
            return iter.0;
        }
    }
    return "null".to_string();
}

async fn register_box(id: String, color: String, boxes: Boxes) {
    boxes.write().await.insert(
        id,
        Entity {
            color,
            position: (0, 0)
        }
    );
}

pub async fn ws_handler(ws: warp::ws::Ws, id: String, clients: Clients, boxes: Boxes) -> Result<impl Reply> {
    let mut client = clients.read().await.get(&id).cloned();
    if client.is_none() {
        clients.write().await.insert(id.clone(), Client {
            word: String::from("None")
        });
        client = clients.read().await.get(&id).cloned();
    }

    match client {
        Some(c) => Ok(ws.on_upgrade(move | socket | ws::client_connection(socket, id, clients, boxes, c))),
        None => Err(warp::reject::not_found())
    }
}

pub async fn health_handler() -> Result<impl Reply> {
    Ok(StatusCode::OK)
}
