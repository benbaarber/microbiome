use nalgebra::{self as na, Point2};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Length/width of the microbiome
const SIZE: i32 = 100;
const BOUND: Point2<f32> = na::point![SIZE as f32, SIZE as f32];

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Cell {
    pos: Point2<f32>,
    mass: i32,
}

impl Cell {
    fn new(pos: Point2<f32>, mass: i32) -> Self {
        Self { pos, mass }
    }
}

impl Default for Cell {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            pos: na::point![
                rng.gen::<f32>() * SIZE as f32,
                rng.gen::<f32>() * SIZE as f32
            ],
            mass: rng.gen_range(8..=12),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Food {
    pos: Point2<f32>,
    mass: i32,
}

impl Food {
    fn new() -> Self {
        Self::default()
    }
}

impl Default for Food {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            pos: na::point![
                rng.gen::<f32>() * SIZE as f32,
                rng.gen::<f32>() * SIZE as f32
            ],
            mass: rng.gen_range(1..=3),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Microbiome {
    agent: Cell,
    npcs: Vec<Cell>,
    food: Vec<Food>,
}

impl Microbiome {
    pub fn new() -> Self {
        Self {
            agent: Cell::default(),
            npcs: std::iter::repeat_with(Cell::default).take(2).collect(),
            food: std::iter::repeat_with(Food::default).take(2).collect(),
        }
    }
}
