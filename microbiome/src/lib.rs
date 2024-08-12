use entities::{Food, NPC};
use invariants::{
    CELL_PERCEPTION_RADIUS, EAT_DIFF, FOOD_PERCEPTION_RADIUS, FOOD_SPAWN_RATE, FPS,
    INITIAL_FOOD_SUPPLY, INITIAL_NUM_NPCS, SIZE,
};
use nalgebra::{point, Point2, Vector2};
use quadtree::{
    shapes::{Circle, Rect, Shape},
    QuadTree,
};
use serde::Serialize;
use util::{QTIndexMassItem, WeightedPoint};

mod entities;
pub mod invariants;
mod util;

type P2 = Point2<f64>;
type V2 = Vector2<f64>;

/// A frame of the microbiome perceived by a cell
#[derive(Debug, Clone)]
pub struct Frame {
    npcs: Vec<WeightedPoint>,
    food: Vec<WeightedPoint>,
}

impl Frame {
    pub fn new(npcs: Vec<WeightedPoint>, food: Vec<WeightedPoint>) -> Self {
        Self { npcs, food }
    }
}

#[derive(Debug, Serialize)]
pub struct Microbiome {
    // agent: Cell,
    boundary: Rect,
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
            boundary,
            npcs: std::iter::repeat_with(NPC::default)
                .take(INITIAL_NUM_NPCS)
                .collect(),
            food,
            elapsed: 0,
        }
    }

    fn get_perceived_frame<'a>(&'a self, pos: P2) -> Frame {
        let area = Circle::new(pos, CELL_PERCEPTION_RADIUS);
        let food = self
            .food
            .query_ref(&area)
            .into_iter()
            .map(|x| WeightedPoint::new(x.pos, x.mass))
            .collect();
        let npcs = self
            .npcs
            .iter()
            .filter(|x| area.contains(&x.pos))
            .map(|x| WeightedPoint::new(x.pos, x.mass))
            .collect(); // O(n) is fine for now because there are not very many NPCs
        Frame::new(npcs, food)
    }

    pub fn step(&mut self) {
        // Spawn food
        if self.elapsed % (FPS / FOOD_SPAWN_RATE) == 0 {
            let food = Food::default();
            self.food.insert(&food);
        }

        // ---- Update NPCs ----
        self.npcs
            .sort_unstable_by(|a, b| b.mass.partial_cmp(&a.mass).unwrap());

        let mut npc_qt = QuadTree::new(self.boundary, 1);
        let npc_ixs_masses = self
            .npcs
            .iter()
            .enumerate()
            .map(|(i, x)| QTIndexMassItem::new(x.pos, x.mass, i))
            .collect::<Vec<_>>();
        npc_qt.insert_many(&npc_ixs_masses);

        let frames = self
            .npcs
            .iter()
            .map(|x| self.get_perceived_frame(x.pos))
            .collect::<Vec<_>>();

        let mut frames = Vec::with_capacity(self.npcs.len());
        for npc in &self.npcs {
            let food_area = Circle::new(npc.pos, FOOD_PERCEPTION_RADIUS);
            let food = self
                .food
                .query_ref(&food_area)
                .into_iter()
                .map(|x| WeightedPoint::new(x.pos, x.mass))
                .collect();
            let npc_area = Circle::new(npc.pos, CELL_PERCEPTION_RADIUS);
            let npcs = npc_qt
                .query_ref(&npc_area)
                .into_iter()
                .map(|x| WeightedPoint::new(x.pos, x.mass))
                .collect();
            frames.push(Frame::new(npcs, food))
        }

        let mut npcs_eaten = vec![false; self.npcs.len()];
        for (i, npc) in self.npcs.iter_mut().enumerate() {
            if npcs_eaten[i] {
                continue;
            };

            let frame = &frames[i];
            npc.step(frame);

            let mut area = Circle::new(npc.pos, npc.radius());

            let eaten = self.food.pop(&area);
            if eaten.len() > 0 {
                npc.mass += eaten.into_iter().map(|f| f.mass).sum::<f64>();
                area.set_radius(npc.radius());
            }

            let eaten = npc_qt.query_filter(&area, |x| x.ix != i && x.mass < npc.mass - EAT_DIFF);
            if eaten.len() > 0 {
                npc.mass += eaten.iter().map(|x| x.mass).sum::<f64>();
                for e in eaten {
                    npcs_eaten[e.ix] = true;
                }
            }
        }

        let mut eaten = npcs_eaten.into_iter();
        self.npcs.retain(|_| !eaten.next().unwrap());
        // ---------------------

        self.elapsed += 1;
    }
}
