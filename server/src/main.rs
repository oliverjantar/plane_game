use messages::ServerMessage;
use warp::{ws::WebSocket, Filter};
mod messages;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let game = warp::path("game")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| ws.on_upgrade(move |socket| user_connected(socket)));

    let status = warp::path!("status").map(move || warp::reply::html("hello"));

    let routes = status.or(game);

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}

async fn user_connected(ws: WebSocket) {
    unimplemented!();
}

async fn send_msg(msg: ServerMessage) {
    let buffer = serde_json::to_vec(&msg).unwrap();
}
