use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct State {
    pub pos: Vec2,
    pub r: f32,
}

#[derive(Clone)]
pub struct RemoteState {
    pub id: usize,
    pub position: Vec2,
    pub rotation: f32,
}

pub enum ServerMessage {
    Welcome(usize),
    GoodBye(usize),
    Update(Vec<RemoteState>),
}

pub enum ClientMessage {
    State(State),
}
