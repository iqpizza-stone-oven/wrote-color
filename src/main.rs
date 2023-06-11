use tokio::{sync::{mpsc, RwLock}, runtime};
use warp::{ws::Message, Filter, Rejection};
use std::{sync::{Arc}, collections::HashMap, convert::Infallible};

mod r#box;

use crate::r#box::Box;

mod game_handler;
mod ws;

pub type Boxes = Arc<RwLock<HashMap<String, Box>>>;
pub type Result<T> = std::result::Result<T, Rejection>;
pub type Clients = Arc<RwLock<HashMap<String, Client>>>;

#[derive(Debug, Clone)]
pub struct Client {
    pub word: String,  // 사용자가 작성한 글자
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>
}

fn main() {
    let runtime = runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let clients: Clients = Arc::new(RwLock::new(HashMap::new()));
        let boxes: Boxes = Arc::new(RwLock::new(HashMap::new()));

        let health_route = warp::path!("health").and_then(game_handler::health_handler);

        let register = warp::path("register");
        let register_routes = register
            .and(warp::post())
            .and(warp::body::json())
            .and(with_boxes(boxes.clone()))
            .and_then(game_handler::register_handler);

        let ws_route = warp::path("ws")
            .and(warp::ws())
            .and(warp::path::param())
            .and(with_clients(clients.clone()))
            .and_then(game_handler::ws_handler);

        let routes = health_route
            .or(register_routes)
            .or(ws_route)
            // .or(publish)
            .with(warp::cors().allow_any_origin());

        warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
    });
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

fn with_boxes(boxes: Boxes) -> impl Filter<Extract = (Boxes,), Error = Infallible> + Clone {
    warp::any().map(move || boxes.clone())
}
