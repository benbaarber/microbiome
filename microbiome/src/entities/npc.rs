use nalgebra::Vector2;
use serde::Serialize;

use crate::{util::random_cell, P2, V2};

#[derive(Debug, Clone, Serialize)]
pub struct NPC {
    pos: P2,
    vel: V2,
    mass: f64,
    radius: f64,
    color: String,
}

impl Default for NPC {
    fn default() -> Self {
        let (pos, mass, radius, color) = random_cell(8..=12);
        Self {
            pos,
            vel: Vector2::zeros(),
            mass,
            radius,
            color,
        }
    }
}

impl NPC {
    pub fn new() -> Self {
        Self::default()
    }
}
