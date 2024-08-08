use constants::{FOOD_SPAWN_RATE, INITIAL_FOOD_SUPPLY, SIZE};
use entities::{Food, NPC};
use nalgebra::{point, Point2, Vector2};
use quadtree::{shapes::Rect, QuadTree};
use serde::Serialize;

pub mod constants;
mod entities;
mod util;

type P2 = Point2<f64>;
type V2 = Vector2<f64>;

#[derive(Debug, Serialize)]
pub struct Microbiome {
    // agent: Cell,
    npcs: Vec<NPC>,
    food: QuadTree<Food>,
    elapsed: u64,
}

impl Microbiome {
    pub fn new() -> Self {
        let boundary = Rect::new(point![0.0, 0.0], point![SIZE, SIZE]);
        let mut food = QuadTree::new(boundary, 10);
        for _ in 0..INITIAL_FOOD_SUPPLY {
            food.insert(&Food::default());
        }

        Self {
            // agent: Cell::default(),
            npcs: std::iter::repeat_with(NPC::default).take(2).collect(),
            food,
            elapsed: 0,
        }
    }

    pub fn step(&mut self) {
        // Spawn food
        if self.elapsed % FOOD_SPAWN_RATE == 0 {
            let food = Food::default();
            self.food.insert(&food);
        }

        self.elapsed += 1;
    }
}
