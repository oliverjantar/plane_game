use futures_util::{FutureExt, StreamExt};
use shared::messages::{ClientMessage, RemoteState, ServerMessage};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{
    ws::{Message, WebSocket},
    Filter,
};

type OutBoundChannel = mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>;

type Users = Arc<RwLock<HashMap<usize, OutBoundChannel>>>;

type States = Arc<RwLock<HashMap<usize, RemoteState>>>;

static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let users = Users::default();
    let states = States::default();

    let arc_users = users.clone();
    let arc_states = states.clone();

    tokio::spawn(async move { update_loop(arc_users, arc_states).await });

    let users = warp::any().map(move || users.clone());
    let states = warp::any().map(move || states.clone());

    let game = warp::path("game")
        .and(warp::ws())
        .and(users)
        .and(states)
        .map(|ws: warp::ws::Ws, users, states| {
            ws.on_upgrade(move |socket| user_connected(socket, users, states))
        });

    let status = warp::path!("status").map(move || warp::reply::html("hello"));

    let routes = status.or(game);

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}

async fn update_loop(users: Users, states: States) {
    loop {
        let states: Vec<RemoteState> = states.read().await.values().cloned().collect();

        if !states.is_empty() {
            for (&uid, tx) in users.read().await.iter() {
                let states = states
                    .iter()
                    .filter_map(|state| {
                        if state.id == uid {
                            None
                        } else {
                            Some(state.clone())
                        }
                    })
                    .collect();

                let states = ServerMessage::Update(states);

                send_msg(tx, &states).await;
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

async fn user_connected(ws: WebSocket, users: Users, states: States) {
    let (ws_sender, mut ws_receiver) = ws.split();

    let send_channel = create_send_channel(ws_sender);
    let my_id = send_welcome(&send_channel).await;

    log::debug!("new user conneted: {}", my_id);

    users.write().await.insert(my_id, send_channel);

    while let Some(result) = ws_receiver.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                log::warn!("websocket err (id={}) '{}'", my_id, e);
                break;
            }
        };

        log::debug!("user sent message: {:?}", msg);

        if let Some(msg) = parse_message(msg) {
            user_message(my_id, msg, &states).await;
        }
    }

    log::debug!("user disconnected: {}", my_id);

    users.write().await.remove(&my_id);
    states.write().await.remove(&my_id);

    broadcast(ServerMessage::GoodBye(my_id), &users).await;
}

async fn send_msg(tx: &OutBoundChannel, msg: &ServerMessage) {
    let buffer = serde_json::to_vec(msg).unwrap();

    let msg = Message::binary(buffer);
    tx.send(Ok(msg)).unwrap();
}

fn create_send_channel(
    ws_sender: futures_util::stream::SplitSink<WebSocket, Message>,
) -> OutBoundChannel {
    let (sender, receiver) = mpsc::unbounded_channel();
    let rx = UnboundedReceiverStream::new(receiver);

    tokio::task::spawn(rx.forward(ws_sender).map(|result| {
        if let Err(e) = result {
            log::error!("websocket send error: {}", e);
        }
    }));

    sender
}

async fn send_welcome(out: &OutBoundChannel) -> usize {
    let id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    let states = ServerMessage::Welcome(id);
    send_msg(out, &states).await;

    id
}

async fn broadcast(msg: ServerMessage, users: &Users) {
    let users = users.read().await;
    for (_, tx) in users.iter() {
        send_msg(tx, &msg).await;
    }
}

fn parse_message(msg: Message) -> Option<ClientMessage> {
    if msg.is_binary() {
        let msg = msg.into_bytes();
        serde_json::from_slice::<ClientMessage>(msg.as_slice()).ok()
    } else {
        None
    }
}

async fn user_message(my_id: usize, msg: ClientMessage, states: &States) {
    match msg {
        ClientMessage::State(state) => {
            let msg = RemoteState {
                id: my_id,
                position: state.pos,
                rotation: state.r,
            };
            states.write().await.insert(msg.id, msg);
        }
    }
}
