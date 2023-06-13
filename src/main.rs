use tokio::{sync::RwLock, runtime};
use warp::{Filter, Rejection, hyper::Method};
use std::{sync::{Arc}, collections::HashMap, convert::Infallible};

mod r#entity;

use crate::entity::Entity;

mod game_handler;
mod ws;
// mod task;

pub type Boxes = Arc<RwLock<HashMap<String, Entity>>>;
pub type Result<T> = std::result::Result<T, Rejection>;
pub type Clients = Arc<RwLock<HashMap<String, Client>>>;

#[derive(Debug, Clone)]
pub struct Client {
    pub word: String,  // 사용자가 작성한 글자
}

fn main() {
    let clients: Clients = Arc::new(RwLock::new(HashMap::new()));
    let boxes: Boxes = Arc::new(RwLock::new(HashMap::new()));

    let runtime = runtime::Runtime::new().unwrap();

    runtime.block_on(async {
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
            .and(with_boxes(boxes.clone()))
            .and_then(game_handler::ws_handler);

        let input_routes = warp::path("input")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_boxes(boxes.clone()))
            .and_then(game_handler::answer_handle);

        let routes = health_route
            .or(register_routes)
            .or(ws_route)
            .or(input_routes)
            .with(warp::cors()
                    .allow_any_origin()
                    .allow_headers(vec!["Access-Control-Allow-Origin", "Origin", "Accept", "X-Requested-With", "Content-Type"])
                    .allow_methods(&[Method::GET, Method::POST]));

        warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
    });
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

fn with_boxes(boxes: Boxes) -> impl Filter<Extract = (Boxes,), Error = Infallible> + Clone {
    warp::any().map(move || boxes.clone())
}
