extern crate ggez;
extern crate rand;

use ggez::event::Keycode;
use ggez::graphics::{DrawMode, Point2};
use ggez::{conf, event, graphics, Context, GameResult};
use rand::prelude::*;
use std::f32;

const PAD_SPEED: f32 = 7.5;
const BALL_SPEED: f32 = 9.0;
const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
    Still,
}

struct MainState {
    pad_one: Pad,
    pad_two: Pad,
    ball: Ball,
    scoreboard: ScoreBoard,
    score_text: graphics::Text,
    font: graphics::Font,
}

// Keeps track of current scores of player pads
struct ScoreBoard {
    pad_one: u32,
    pad_two: u32,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let font = graphics::Font::default_font().unwrap();
        let mut s = MainState {
            pad_one: Pad::new((20.0, 275.0), (10.0, 50.0)),
            pad_two: Pad::new((770.0, 275.0), (10.0, 50.0)),
            ball: Ball::new((400.0, 300.0), (10.0, 10.0)),
            scoreboard: ScoreBoard {
                pad_one: 0,
                pad_two: 0,
            },
            score_text: graphics::Text::new(ctx, "0 : 0", &font).unwrap(),
            font,
        };
        s.ball.spawn_in_middle(); // Spawn the ball in the middle of the screen
        Ok(s)
    }

    fn update_score_text(&mut self, ctx: &mut Context) {
        self.score_text = graphics::Text::new(
            ctx,
            &format!("{} : {}", self.scoreboard.pad_one, self.scoreboard.pad_two),
            &self.font,
        ).unwrap();
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.pad_one.update();
        self.pad_two.update();
        let score_changed = self.ball
            .update(&[&self.pad_one, &self.pad_two], &mut self.scoreboard);
        if score_changed {
            self.update_score_text(ctx);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        graphics::rectangle(
            ctx,
            DrawMode::Fill,
            to_rectangle(self.pad_one.pos, self.pad_one.size),
        )?;
        graphics::rectangle(
            ctx,
            DrawMode::Fill,
            to_rectangle(self.pad_two.pos, self.pad_two.size),
        )?;
        graphics::rectangle(
            ctx,
            DrawMode::Fill,
            to_rectangle(self.ball.pos, self.ball.size),
        )?;
        //
        // Draw the score text in the top middle of the screen
        graphics::draw(
            ctx,
            &self.score_text,
            Point2::new(
                SCREEN_WIDTH as f32 / 2.0 - self.score_text.width() as f32 / 2.0,
                10.0,
            ),
            0.0,
        ).unwrap();
        graphics::present(ctx);
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: Keycode,
        _keymod: event::Mod,
        _repeat: bool,
    ) {
        match keycode {
            Keycode::Up => self.pad_two.direction = Direction::Up,
            Keycode::Down => self.pad_two.direction = Direction::Down,
            Keycode::W => self.pad_one.direction = Direction::Up,
            Keycode::S => self.pad_one.direction = Direction::Down,
            _ => {}
        }
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut Context,
        keycode: Keycode,
        _keymod: event::Mod,
        _repeat: bool,
    ) {
        match keycode {
            Keycode::Up => {
                if self.pad_two.direction == Direction::Up {
                    self.pad_two.direction = Direction::Still
                }
            }
            Keycode::Down => {
                if self.pad_two.direction == Direction::Down {
                    self.pad_two.direction = Direction::Still
                }
            }
            Keycode::W => {
                if self.pad_one.direction == Direction::Up {
                    self.pad_one.direction = Direction::Still
                }
            }
            Keycode::S => {
                if self.pad_one.direction == Direction::Down {
                    self.pad_one.direction = Direction::Still
                }
            }
            _ => {}
        }
    }
}

// Turns (x, y) and (width, height) f32 tuples into a graphics::Rect
fn to_rectangle(pos: (f32, f32), size: (f32, f32)) -> graphics::Rect {
    graphics::Rect::new(pos.0, pos.1, size.0, size.1)
}

// Player pad
struct Pad {
    pos: (f32, f32),
    size: (f32, f32),
    direction: Direction,
}

impl Pad {
    fn new(pos: (f32, f32), size: (f32, f32)) -> Pad {
        Pad {
            pos,
            size,
            direction: Direction::Still,
        }
    }

    fn update(&mut self) {
        self.movement_update();
    }

    fn movement_update(&mut self) {
        match self.direction {
            Direction::Up => self.pos = offset_pos(self.pos, (0.0, -PAD_SPEED)),
            Direction::Down => self.pos = offset_pos(self.pos, (0.0, PAD_SPEED)),
            _ => {}
        }

        // Clamp to borders
        if self.pos.1 < 0.0 - self.size.1 / 2.0 {
            self.pos.1 = 0.0 - self.size.1 / 2.0;
        } else if self.pos.1 + self.size.1 > SCREEN_HEIGHT as f32 + self.size.1 / 2.0 {
            self.pos.1 = SCREEN_HEIGHT as f32 - self.size.1 / 2.0;
        }
    }
}

// Game ball
struct Ball {
    pos: (f32, f32),
    size: (f32, f32),
    angle: f32,
    direction: Direction,
    last_hit_index: u32,
}

impl Ball {
    fn new(pos: (f32, f32), size: (f32, f32)) -> Ball {
        Ball {
            pos,
            size,
            angle: 90.0,
            direction: Direction::Right,
            last_hit_index: 0,
        }
    }

    fn update(&mut self, pads: &[&Pad], scoreboard: &mut ScoreBoard) -> bool {
        let player_index_hit = self.collision_update(pads);
        if let Some(index) = player_index_hit {
            self.last_hit_index = index;
        }

        // Move in direction of angle
        let x = self.pos.0 + (self.angle.to_radians().sin() * BALL_SPEED);
        let y = self.pos.1 + (self.angle.to_radians().cos() * BALL_SPEED);

        self.pos = (x, y);

        // Check if ball is out of bounds
        if (x as i32) > SCREEN_WIDTH as i32 + 50 || ((x + self.size.0) as i32) < -50 {
            match self.last_hit_index {
                0 => scoreboard.pad_one += 1,
                _ => scoreboard.pad_two += 1,
            }
            self.spawn_in_middle();
            return true;
        } else if (y as i32) > SCREEN_HEIGHT as i32 + 20 || ((y + self.size.1) as i32) < -20 {
            match self.last_hit_index {
                0 => scoreboard.pad_two += 1,
                _ => scoreboard.pad_one += 1,
            }
            self.spawn_in_middle();
            return true;
        }
        false
    }

    // Spawns the ball in the middle of the screen, with a random initial angle and direction
    fn spawn_in_middle(&mut self) {
        let x = SCREEN_WIDTH as f32 / 2.0 - self.size.0 / 2.0;
        let y = SCREEN_HEIGHT as f32 / 2.0 - self.size.1 / 2.0;

        self.pos = (x, y);
        self.angle = thread_rng().gen_range(60.0, 120.0);
        if random() {
            self.direction = Direction::Left;
            self.angle = -self.angle;
            self.last_hit_index = 1;
        } else {
            self.direction = Direction::Right;
            self.last_hit_index = 0;
        }
    }

    // Check if a collision has happened and move the ball accordingly
    // If collided, returns Some(index) of the player pad that was collided with, otherwise None
    fn collision_update(&mut self, pads: &[&Pad]) -> Option<u32> {
        let mut collider_pad: Option<&Pad> = None;
        let mut player_index_hit = None;

        // Check each player pad to see if collision happened
        for (i, pad) in pads.iter().enumerate() {
            if self.pos.0 + self.size.0 > pad.pos.0
                && self.pos.0 < pad.pos.0 + pad.size.0
                && self.pos.1 + self.size.1 > pad.pos.1
                && self.pos.1 < pad.pos.1 + pad.size.1
            {
                collider_pad = Some(&pad);
                player_index_hit = Some(i as u32);
                break;
            }
        }

        // If collided, handle collision by setting new angle
        if let Some(pad) = collider_pad {
            self.angle = 112.5 - 45.0 * ((self.pos.1 - pad.pos.1) / pad.size.1);

            if self.direction == Direction::Right {
                self.angle = -self.angle; // Reverse angle for this direction
                self.direction = Direction::Left;
                self.pos.0 = pad.pos.0 - self.size.0; // Move out of pad
            } else {
                self.direction = Direction::Right;
                self.pos.0 = pad.pos.0 + pad.pos.0; // Move out of pad
            }
        }
        
        // Return index of player that hit the ball
        player_index_hit
    }
}

fn offset_pos(pos: (f32, f32), offset: (f32, f32)) -> (f32, f32) {
    (pos.0 + offset.0, pos.1 + offset.1)
}

fn main() {
    println!("Welcome to Pong - Written in Rust");

    // Rendering
    let ctx = &mut ggez::ContextBuilder::new("pong", "skuzzi")
        .window_setup(ggez::conf::WindowSetup::default().title("Pong"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_WIDTH, SCREEN_HEIGHT))
        .build()
        .expect("Failed to build ggez context");

    let state = &mut MainState::new(ctx).unwrap();
    event::run(ctx, state).unwrap();
}
