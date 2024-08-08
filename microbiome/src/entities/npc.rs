use nalgebra::vector;
use quadtree::Point;
use serde::{ser::SerializeStruct, Serialize};

use crate::{
    invariants::{radius, speed, SIZE},
    util::random_cell,
    P2, V2,
};

#[derive(Debug, Clone)]
pub enum NPCKind {
    Linear(V2),
}

#[derive(Debug, Clone)]
pub struct NPC {
    pub pos: P2,
    pub mass: f64,
    pub color: String,
    pub kind: NPCKind,
}

impl Default for NPC {
    fn default() -> Self {
        let (pos, mass, color) = random_cell(20..=30);
        Self {
            pos,
            mass,
            color,
            kind: NPCKind::Linear((V2::new_random() - vector![0.5, 0.5]).normalize()),
        }
    }
}

impl NPC {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn radius(&self) -> f64 {
        radius(self.mass)
    }

    pub fn speed(&self) -> f64 {
        speed(self.mass)
    }

    pub fn step(&mut self) {
        let speed = self.speed();
        let limit = self.radius() * 0.9;
        match &mut self.kind {
            NPCKind::Linear(dir) => {
                let next_pos = self.pos + (*dir * speed);
                if next_pos.x <= limit {
                    dir.x = dir.x.abs();
                } else if next_pos.x >= SIZE - limit {
                    dir.x = -dir.x.abs();
                }
                if next_pos.y <= limit {
                    dir.y = dir.y.abs();
                } else if next_pos.y >= SIZE - limit {
                    dir.y = -dir.y.abs();
                }
                self.pos += *dir * speed;
            }
        }
    }
}

impl Point for NPC {
    fn point(&self) -> P2 {
        self.pos
    }
}

impl Serialize for NPC {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("NPC", 5)?;
        state.serialize_field("pos", &self.pos)?;
        state.serialize_field("radius", &self.radius())?;
        state.serialize_field("mass", &self.mass)?;
        state.serialize_field("pos", &self.pos)?;
        state.serialize_field("color", &self.color)?;
        state.end()
    }
}
