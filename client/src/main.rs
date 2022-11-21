mod ws;

use macroquad::prelude::*;
use shared::messages::ServerMessage;
use ws::Connection;

#[macroquad::main("game")]
async fn main() {
    let mut connection = Connection::new();
    connection.connect("ws://localhost:3030/game");

    let mut game = Game::new().await;
    loop {
        if let Some(msg) = connection.poll() {
            let msg: ServerMessage =
                serde_json::from_slice(msg.as_slice()).expect("deserialization failed");
            game.handle_message(msg);
        }

        game.update();
        game.draw();
        if game.quit {
            return;
        }
        next_frame().await
    }
}

pub struct Game {
    pub quit: bool,
    pub player_state: PlayerState,
    pub texture: Texture2D,
}

impl Game {
    pub async fn new() -> Self {
        let texture = load_texture("assets/plane.png").await.unwrap();
        Self {
            player_state: PlayerState {
                id: 0,
                position: Vec2::new(100f32, 100f32),
                rotation: 0f32,
            },
            texture,
            quit: false,
        }
    }

    pub fn handle_message(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::Welcome(id) => {
                self.player_state.id = id;
            }
            ServerMessage::GoodBye(_) => {
                unimplemented!();
            }
            ServerMessage::Update(_) => {
                unimplemented!();
            }
        }
    }

    pub fn update(&mut self) {
        if is_key_down(KeyCode::Escape) {
            self.quit = true;
        }

        const ROT_SPEED: f32 = 0.015;
        const SPEED: f32 = 0.6;

        if is_key_down(KeyCode::Right) {
            self.player_state.rotation += ROT_SPEED;
        }
        if is_key_down(KeyCode::Left) {
            self.player_state.rotation -= ROT_SPEED;
        }

        self.player_state.position += Self::vec2_from_angle(self.player_state.rotation) * SPEED;

        if self.player_state.position.x > screen_width() {
            self.player_state.position.x = -self.texture.width();
        } else if self.player_state.position.x < -self.texture.width() {
            self.player_state.position.x = screen_width();
        }

        if self.player_state.position.y > screen_height() {
            self.player_state.position.y = -self.texture.height();
        } else if self.player_state.position.y < -self.texture.height() {
            self.player_state.position.y = screen_height();
        }
    }

    pub fn draw(&self) {
        clear_background(color_u8!(255, 255, 255, 255));

        // draw_poly_lines(
        //     self.player_state.position.x,
        //     self.player_state.position.y,
        //     3,
        //     10.,
        //     self.player_state.rotation * 180. / std::f32::consts::PI - 90.,
        //     2.,
        //     BLACK,
        // );

        draw_texture_ex(
            self.texture,
            self.player_state.position.x,
            self.player_state.position.y,
            WHITE,
            DrawTextureParams {
                rotation: self.player_state.rotation,
                ..Default::default()
            },
        );

        Self::draw_box(Vec2::new(400f32, 200f32), Vec2::new(50f32, 20f32));
    }

    fn draw_box(pos: Vec2, size: Vec2) {
        let dimension = size * 2.;
        let upper_left = pos - size;

        draw_rectangle(upper_left.x, upper_left.y, dimension.x, dimension.y, BLACK);
    }

    //calculates a directional vector using our current rotation angle value
    fn vec2_from_angle(angle: f32) -> Vec2 {
        let angle = angle - std::f32::consts::FRAC_PI_2;
        Vec2::new(angle.cos(), angle.sin())
    }
}

#[derive(Default)]
pub struct PlayerState {
    pub id: usize,
    pub position: Vec2,
    pub rotation: f32,
}
