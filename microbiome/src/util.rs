use nalgebra::{self as na, point};
use quadtree::Point;
use rand::{distributions::uniform::SampleRange, Rng};
use random_color::RandomColor;

use crate::{invariants::SIZE, P2};

/// Generate a random position and mass from a given range
///
/// **Returns** (pos, mass, color)
pub fn random_cell(mass_range: impl SampleRange<i32>) -> (P2, f64, String) {
    let mut rng = rand::thread_rng();
    let mass = rng.gen_range(mass_range) as f64;
    let pos = na::point![
        rng.gen::<f64>() * SIZE as f64,
        rng.gen::<f64>() * SIZE as f64
    ];
    let color = random_color();
    (pos, mass, color)
}

/// Generate a random hex color
pub fn random_color() -> String {
    RandomColor::new().to_hex()
}

pub fn restrict_cell_to_bounds(point: P2, radius: f64) -> P2 {
    point![
        point.x.clamp(radius, SIZE - radius),
        point.y.clamp(radius, SIZE - radius),
    ]
}

#[derive(Debug, Clone, Copy)]
pub struct WeightedPoint {
    pub pos: P2,
    pub mass: f64,
}

impl WeightedPoint {
    pub fn new(pos: P2, mass: f64) -> Self {
        Self { pos, mass }
    }
}

impl Point for WeightedPoint {
    fn point(&self) -> P2 {
        self.pos
    }
}

/// For populating a QuadTree with indices instead of actual items
#[derive(Debug, Clone, Copy)]
pub struct QTIndexMassItem {
    pub pos: P2,
    pub mass: f64,
    pub ix: usize,
}

impl QTIndexMassItem {
    pub fn new(pos: P2, mass: f64, ix: usize) -> Self {
        Self { pos, mass, ix }
    }
}

impl Point for QTIndexMassItem {
    fn point(&self) -> P2 {
        self.pos
    }
}
