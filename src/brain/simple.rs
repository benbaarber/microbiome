use macroquad::prelude::*;

use super::Action;
use crate::invariants::*;
use crate::simulation::{EntityId, EntityType, World};

// AI behavior priority (highest to lowest):
//   1. FLEE: If a threat (cell/mote 10%+ bigger) is within danger_range, eject mass
//      towards it to propel away. Urgency scales with proximity. Gaze looks away.
//   2. HUNT: If prey (cell/mote 10%+ smaller AND at least 10% of hunter's mass) is
//      within chase_range, chase aggressively. Ignores tiny prey not worth the cost.
//      Gaze looks towards prey.
//   3. FORAGE: If food is within extended range, pursue it gently.
//      Prefers closer/larger food (scored by distance/mass ratio). Gaze looks towards food.
//   4. WANDER: Otherwise, occasionally eject mass in a random direction.
//      Gaze drifts towards movement direction.
pub fn think(world: &World, id: EntityId, dt: f32) -> Action {
    let my_pos = world.positions[id];
    let my_mass = world.masses[id];
    let my_radius = mass_to_radius(my_mass);

    let n = world.types.len();
    let mut nearest_threat: Option<(f32, Vec2)> = None;
    let mut nearest_food: Option<(f32, Vec2, f32)> = None;
    let mut nearest_prey: Option<(f32, Vec2)> = None;

    for other in 0..n {
        if other == id || world.types[other] == EntityType::None {
            continue;
        }
        let other_pos = world.positions[other];
        let other_mass = world.masses[other];
        let dist = (other_pos - my_pos).length();

        let other_type = world.types[other];
        if other_type == EntityType::Food {
            let score = dist / other_mass;
            if nearest_food.is_none() || score < nearest_food.unwrap().0 / nearest_food.unwrap().2 {
                nearest_food = Some((dist, other_pos, other_mass));
            }
        } else if other_type == EntityType::Cell || other_type == EntityType::Mote {
            if other_mass > my_mass * 1.1 {
                let dominated_dist = dist - mass_to_radius(other_mass) - my_radius;
                if nearest_threat.is_none() || dominated_dist < nearest_threat.unwrap().0 {
                    nearest_threat = Some((dominated_dist, other_pos));
                }
            } else if other_mass < my_mass * 0.9 && other_mass > my_mass * 0.1 {
                if nearest_prey.is_none() || dist < nearest_prey.unwrap().0 {
                    nearest_prey = Some((dist, other_pos));
                }
            }
        }
    }

    let danger_range = my_radius * 8.0;
    let chase_range = my_radius * 12.0;

    if let Some((dist, threat_pos)) = nearest_threat {
        if dist < danger_range {
            let flee_dir = (threat_pos - my_pos).normalize_or_zero();
            let urgency = 1.0 - (dist / danger_range).max(0.0);
            let gaze_strength = 0.5 + urgency * 0.5;
            let eject = if rand::gen_range(0.0, 1.0) < (0.05 * urgency + 0.01) * dt * 60.0 {
                Some((flee_dir, 0.2 + urgency * 0.3))
            } else {
                None
            };
            return Action {
                eject,
                gaze: flee_dir * gaze_strength,
                gaze_speed: 5.0,
            };
        }
    }

    if let Some((dist, prey_pos)) = nearest_prey {
        if dist < chase_range {
            let look_dir = (prey_pos - my_pos).normalize_or_zero();
            let chase_dir = (my_pos - prey_pos).normalize_or_zero();
            let eject = if rand::gen_range(0.0, 1.0) < 0.04 * dt * 60.0 {
                Some((chase_dir, rand::gen_range(0.2, 0.4)))
            } else {
                None
            };
            return Action {
                eject,
                gaze: look_dir * 0.6,
                gaze_speed: 5.0,
            };
        }
    }

    if let Some((dist, food_pos, _)) = nearest_food {
        if dist < chase_range * 1.5 {
            let look_dir = (food_pos - my_pos).normalize_or_zero();
            let chase_dir = (my_pos - food_pos).normalize_or_zero();
            let eject = if rand::gen_range(0.0, 1.0) < 0.012 * dt * 60.0 {
                Some((chase_dir, rand::gen_range(0.1, 0.2)))
            } else {
                None
            };
            return Action {
                eject,
                gaze: look_dir * 0.3,
                gaze_speed: 5.0,
            };
        }
    }

    let vel = world.velocities[id];
    let gaze = if vel.length_squared() > 1.0 {
        vel.normalize() * 0.15
    } else {
        Vec2::ZERO
    };
    let eject = if rand::gen_range(0.0, 1.0) < 0.005 * dt * 60.0 {
        let angle = rand::gen_range(0.0, std::f32::consts::TAU);
        let dir = vec2(angle.cos(), angle.sin());
        Some((dir, rand::gen_range(0.1, 0.2)))
    } else {
        None
    };

    Action {
        eject,
        gaze,
        gaze_speed: 2.0,
    }
}
