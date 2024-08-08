use entities::{Food, NPC};
use invariants::{FOOD_SPAWN_RATE, FPS, INITIAL_FOOD_SUPPLY, INITIAL_NUM_NPCS, SIZE};
use nalgebra::{point, Point2, Vector2};
use quadtree::{
    shapes::{Circle, Rect},
    QuadTree,
};
use serde::Serialize;
use util::QTIndexMassItem;

mod entities;
pub mod invariants;
mod util;

type P2 = Point2<f64>;
type V2 = Vector2<f64>;

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

        let mut npcs_eaten = vec![false; self.npcs.len()];
        for (i, npc) in self.npcs.iter_mut().enumerate() {
            if npcs_eaten[i] {
                continue;
            };

            npc.step();
            let mut area = Circle::new(npc.pos, npc.radius());

            let eaten = self.food.pop(&area);
            if eaten.len() > 0 {
                npc.mass += eaten.into_iter().map(|f| f.mass).sum::<f64>();
                area.set_radius(npc.radius());
            }

            let mut eaten = npc_qt.pop(&area);
            eaten.retain(|x| x.ix != i);
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
