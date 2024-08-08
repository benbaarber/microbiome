use quadtree::Point;
use serde::Serialize;

use crate::{util::random_cell, P2};

#[derive(Debug, Clone, Serialize)]
pub struct Food {
    pos: P2,
    mass: f64,
    radius: f64,
    color: String,
}

impl Food {
    fn new() -> Self {
        Self::default()
    }
}

impl Default for Food {
    fn default() -> Self {
        let (pos, mass, radius, color) = random_cell(1..=3);
        Self {
            pos,
            mass,
            radius,
            color,
        }
    }
}

impl Point for Food {
    fn point(&self) -> P2 {
        self.pos
    }
}
