use nalgebra::{self as na, vector};
use quadtree::Point;
use serde::{ser::SerializeStruct, Serialize};

use crate::{
    invariants::{radius, speed, EAT_DIFF, SIZE},
    util::{random_cell, restrict_cell_to_bounds, WeightedPoint},
    Frame, P2, V2,
};

#[derive(Debug, Clone)]
pub enum NPCKind {
    Linear { dir: V2 },
    Advanced { dir: V2 },
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
            // kind: NPCKind::Linear {
            //     dir: (V2::new_random() - vector![0.5, 0.5]).normalize(),
            // },
            kind: NPCKind::Advanced {
                dir: (V2::new_random() - vector![0.5, 0.5]).normalize(),
            },
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

    pub fn step(&mut self, frame: &Frame) {
        let speed = self.speed();
        let limit = self.radius() * 0.9;
        let dir = match &mut self.kind {
            NPCKind::Linear { dir } => {
                let next_pos = self.pos + *dir * speed;
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
                *dir
            }
            NPCKind::Advanced { dir } => {
                let mut prey = &WeightedPoint::new(P2::origin(), 0.0);
                let mut predators = Vec::with_capacity(frame.npcs.len());
                for npc in &frame.npcs {
                    if self.mass > npc.mass + EAT_DIFF {
                        if npc.mass > prey.mass {
                            prey = npc;
                        }
                    } else if self.mass < npc.mass - EAT_DIFF {
                        predators.push(npc);
                    }
                }

                if predators.len() > 0 {
                    // Find the unweighted center of all predators in the vicinity and run away from that
                    let sum = predators.into_iter().map(|x| x.pos - self.pos).sum::<V2>();
                    *dir = -sum.normalize();
                } else if prey.mass > 0.0 {
                    // Find the largest prey and chase him
                    *dir = (prey.pos - self.pos).normalize()
                } else if frame.food.len() > 0 {
                    let mut food = &frame.food[0];
                    let mut min_dist = f64::MAX;
                    for f in &frame.food {
                        let dist = na::distance(&self.pos, &f.pos);
                        if dist < min_dist {
                            food = f;
                            min_dist = dist;
                            if dist < 2.0 {
                                break;
                            }
                        }
                    }
                    *dir = (food.pos - self.pos).normalize()
                }

                *dir
            }
        };

        self.pos += dir * speed;
        self.pos = restrict_cell_to_bounds(self.pos, self.radius());
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
