use macroquad::prelude::*;

pub mod ctrnn;
pub mod simple;

pub struct Action {
    pub eject: Option<(Vec2, f32)>,
    pub gaze: Vec2,
    pub gaze_speed: f32,
}
