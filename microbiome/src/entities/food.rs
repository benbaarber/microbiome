use quadtree::Point;
use serde::{ser::SerializeStruct, Serialize};

use crate::{invariants::radius, util::random_cell, P2};

#[derive(Debug, Clone)]
pub struct Food {
    pub pos: P2,
    pub mass: f64,
    pub color: String,
}

impl Default for Food {
    fn default() -> Self {
        let (pos, mass, color) = random_cell(1..=3);
        Self { pos, mass, color }
    }
}

impl Food {
    fn new() -> Self {
        Self::default()
    }

    fn radius(&self) -> f64 {
        radius(self.mass)
    }
}

impl Point for Food {
    fn point(&self) -> P2 {
        self.pos
    }
}

impl Serialize for Food {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Food", 4)?;
        state.serialize_field("pos", &self.pos)?;
        state.serialize_field("radius", &self.radius())?;
        state.serialize_field("mass", &self.mass)?;
        state.serialize_field("color", &self.color)?;
        state.end()
    }
}
