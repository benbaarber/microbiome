use macroquad::prelude::*;

use crate::invariants::*;

pub type EntityId = usize;

const PLAYER_COLOR: Color = Color::new(0.3, 0.7, 1.0, 1.0);

#[derive(Clone)]
pub struct WorldConfig {
    pub player_mass: f32,
    pub cell_count: usize,
    pub cell_mass: f32,
    pub cell_spawn_radius_min: f32,
    pub cell_spawn_radius_max: f32,
    pub food_count: usize,
    pub food_spawn_radius: f32,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            player_mass: 3000.0,
            cell_count: 20,
            cell_mass: 500.0,
            cell_spawn_radius_min: BOUNDARY_RADIUS * 0.25,
            cell_spawn_radius_max: BOUNDARY_RADIUS * 0.75,
            food_count: 30,
            food_spawn_radius: BOUNDARY_RADIUS * 0.9,
        }
    }
}

fn random_cell_color() -> Color {
    let hue = rand::gen_range(0.0, 1.0);
    let (r, g, b) = hsv_to_rgb(hue, 0.7, 0.9);
    Color::new(r, g, b, 1.0)
}

fn random_point_in_circle(min_radius: f32, max_radius: f32) -> Vec2 {
    let angle = rand::gen_range(0.0, std::f32::consts::TAU);
    let dist = rand::gen_range(min_radius, max_radius);
    vec2(angle.cos() * dist, angle.sin() * dist)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntityType {
    None,
    Cell,
    Mote,
    Food,
}

#[derive(Clone, Copy)]
pub struct Collision {
    pub contact_pos: Vec2,
    pub normal: Vec2, // Points from smaller to larger
    pub smaller_radius: f32,
    pub larger_color: Color,
}

#[derive(Clone, Copy)]
pub struct Explosion {
    pub pos: Vec2,
    pub radius: f32,
    pub color: Color,
    pub timer: f32,
}

#[derive(Clone, Copy)]
pub struct Awakening {
    pub entity: EntityId,
    pub color: Color,
    pub timer: f32,
}

pub struct World {
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub masses: Vec<f32>,
    pub colors: Vec<Color>,
    pub types: Vec<EntityType>,
    pub gazes: Vec<Vec2>, // Normalized direction the entity is "looking"
    pub collisions: Vec<Collision>,
    pub explosions: Vec<Explosion>,
    pub awakenings: Vec<Awakening>,
    pub deaths: Vec<(EntityId, EntityId)>, // (victim, killer)

    free_ids: Vec<EntityId>,
    food_spawn_timer: f32,
}

impl World {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            velocities: Vec::new(),
            masses: Vec::new(),
            colors: Vec::new(),
            types: Vec::new(),
            gazes: Vec::new(),
            collisions: Vec::new(),
            explosions: Vec::new(),
            awakenings: Vec::new(),
            deaths: Vec::new(),
            free_ids: Vec::new(),
            food_spawn_timer: 0.0,
        }
    }

    pub fn from_config(config: &WorldConfig) -> (Self, EntityId) {
        let mut world = Self::new();

        let player = world.spawn_cell(Vec2::ZERO, config.player_mass, PLAYER_COLOR);

        for _ in 0..config.cell_count {
            let pos =
                random_point_in_circle(config.cell_spawn_radius_min, config.cell_spawn_radius_max);
            world.spawn_cell(pos, config.cell_mass, random_cell_color());
        }

        for _ in 0..config.food_count {
            let pos = random_point_in_circle(0.0, config.food_spawn_radius);
            world.spawn_food(pos);
        }

        (world, player)
    }

    fn alloc(
        &mut self,
        pos: Vec2,
        vel: Vec2,
        mass: f32,
        color: Color,
        entity_type: EntityType,
    ) -> EntityId {
        if let Some(id) = self.free_ids.pop() {
            self.positions[id] = pos;
            self.velocities[id] = vel;
            self.masses[id] = mass;
            self.colors[id] = color;
            self.types[id] = entity_type;
            self.gazes[id] = Vec2::ZERO;
            id
        } else {
            let id = self.positions.len();
            self.positions.push(pos);
            self.velocities.push(vel);
            self.masses.push(mass);
            self.colors.push(color);
            self.types.push(entity_type);
            self.gazes.push(Vec2::ZERO);
            id
        }
    }

    pub fn spawn_cell(&mut self, pos: Vec2, mass: f32, color: Color) -> EntityId {
        self.alloc(pos, Vec2::ZERO, mass, color, EntityType::Cell)
    }

    pub fn spawn_mote(&mut self, pos: Vec2, vel: Vec2, mass: f32, color: Color) -> EntityId {
        self.alloc(pos, vel, mass, color, EntityType::Mote)
    }

    pub fn spawn_food(&mut self, pos: Vec2) -> EntityId {
        self.alloc(
            pos,
            Vec2::ZERO,
            FOOD_MASS,
            Color::new(0.2, 0.8, 0.2, 1.0),
            EntityType::Food,
        )
    }

    fn remove_entity(&mut self, id: EntityId) {
        if id < self.types.len() && self.types[id] != EntityType::None {
            self.types[id] = EntityType::None;
            self.free_ids.push(id);
        }
    }

    pub fn is_cell(&self, id: EntityId) -> bool {
        id < self.types.len() && self.types[id] == EntityType::Cell
    }

    pub fn count_by_type(&self, entity_type: EntityType) -> usize {
        self.types.iter().filter(|&&t| t == entity_type).count()
    }

    pub fn entity_at_point(
        &self,
        point: Vec2,
        type_filter: impl Fn(EntityType) -> bool,
    ) -> Option<EntityId> {
        let mut best: Option<(EntityId, f32)> = None;
        for i in 0..self.types.len() {
            if !type_filter(self.types[i]) {
                continue;
            }
            let pos = self.positions[i];
            let radius = mass_to_radius(self.masses[i]);
            let dist = (point - pos).length();
            if dist < radius && (best.is_none() || dist < best.unwrap().1) {
                best = Some((i, dist));
            }
        }
        best.map(|(id, _)| id)
    }

    pub fn eject_mass(&mut self, id: EntityId, direction: Vec2, amount: f32) {
        if !self.is_cell(id) {
            return;
        }

        let amount = amount.clamp(0.01, 1.0);

        let dir = if direction.length_squared() > 0.0 {
            direction.normalize()
        } else {
            return;
        };

        let cell_mass = self.masses[id];
        let cell_pos = self.positions[id];
        let cell_color = self.colors[id];

        let ejected_mass = cell_mass * amount * EJECT_MASS_RATIO;
        if ejected_mass < 0.1 || cell_mass - ejected_mass < MIN_CELL_MASS {
            return;
        }

        let eject_velocity = dir * EJECT_SPEED;
        let remaining_mass = cell_mass - ejected_mass;

        self.masses[id] = remaining_mass;

        self.velocities[id] -= (ejected_mass / remaining_mass) * eject_velocity;

        let cell_radius = mass_to_radius(cell_mass);
        let mote_pos = cell_pos + dir * (cell_radius + 5.0);
        let mote_color = Color::new(
            cell_color.r * 0.7,
            cell_color.g * 0.7,
            cell_color.b * 0.7,
            cell_color.a,
        );
        self.spawn_mote(mote_pos, eject_velocity, ejected_mass, mote_color);
    }

    pub fn explode(&mut self, id: EntityId) {
        if id >= self.types.len() || self.types[id] == EntityType::None {
            return;
        }

        let pos = self.positions[id];
        let mass = self.masses[id];
        let color = self.colors[id];
        let radius = mass_to_radius(mass);

        let fragment_mass = mass / 8.0;
        let fragment_color = Color::new(color.r * 0.8, color.g * 0.8, color.b * 0.8, color.a);

        for i in 0..8 {
            let angle = (i as f32 / 8.0) * std::f32::consts::TAU;
            let dir = vec2(angle.cos(), angle.sin());
            let fragment_pos = pos + dir * (radius + 5.0);
            let fragment_vel = dir * EJECT_SPEED * 0.8;
            self.spawn_mote(fragment_pos, fragment_vel, fragment_mass, fragment_color);
        }

        self.explosions.push(Explosion {
            pos,
            radius,
            color,
            timer: EXPLOSION_DURATION,
        });

        self.remove_entity(id);
    }

    fn maybe_spawn_food(&mut self, dt: f32) {
        self.food_spawn_timer += dt;
        if self.food_spawn_timer > 1.0 / FOOD_SPAWN_RATE
            && self.count_by_type(EntityType::Food) < MAX_FOOD
        {
            self.food_spawn_timer = 0.0;
            let pos = random_point_in_circle(0.0, BOUNDARY_RADIUS * 0.9);
            self.spawn_food(pos);
        }
    }

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
    fn run_ai(&mut self, player_id: Option<EntityId>, dt: f32) {
        let n = self.types.len();
        for id in 0..n {
            if Some(id) == player_id || self.types[id] != EntityType::Cell {
                continue;
            }

            let my_pos = self.positions[id];
            let my_mass = self.masses[id];
            let my_radius = mass_to_radius(my_mass);

            let mut nearest_threat: Option<(f32, Vec2)> = None;
            let mut nearest_food: Option<(f32, Vec2, f32)> = None;
            let mut nearest_prey: Option<(f32, Vec2)> = None;

            for other in 0..n {
                if other == id || self.types[other] == EntityType::None {
                    continue;
                }
                let other_pos = self.positions[other];
                let other_mass = self.masses[other];
                let dist = (other_pos - my_pos).length();

                let other_type = self.types[other];
                if other_type == EntityType::Food {
                    let score = dist / other_mass;
                    if nearest_food.is_none()
                        || score < nearest_food.unwrap().0 / nearest_food.unwrap().2
                    {
                        nearest_food = Some((dist, other_pos, other_mass));
                    }
                } else if other_type == EntityType::Cell || other_type == EntityType::Mote {
                    // Consider both cells and motes as potential threats or prey
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

            // Determine target gaze direction based on AI state
            let target_gaze: Vec2;

            if let Some((dist, threat_pos)) = nearest_threat {
                if dist < danger_range {
                    let flee_dir = (threat_pos - my_pos).normalize_or_zero();
                    let urgency = 1.0 - (dist / danger_range).max(0.0);
                    // Look AWAY from threat - full strength when urgent
                    let gaze_strength = 0.5 + urgency * 0.5;
                    target_gaze = -flee_dir * gaze_strength;
                    if rand::gen_range(0.0, 1.0) < (0.05 * urgency + 0.01) * dt * 60.0 {
                        self.eject_mass(id, flee_dir, 0.2 + urgency * 0.3);
                    }
                    self.gazes[id] = self.gazes[id].lerp(target_gaze, 5.0 * dt);
                    continue;
                }
            }

            if let Some((dist, prey_pos)) = nearest_prey {
                if dist < chase_range {
                    // Look TOWARDS prey - medium strength
                    let look_dir = (prey_pos - my_pos).normalize_or_zero();
                    target_gaze = look_dir * 0.6;
                    let chase_dir = (my_pos - prey_pos).normalize_or_zero();
                    if rand::gen_range(0.0, 1.0) < 0.04 * dt * 60.0 {
                        self.eject_mass(id, chase_dir, rand::gen_range(0.2, 0.4));
                    }
                    self.gazes[id] = self.gazes[id].lerp(target_gaze, 5.0 * dt);
                    continue;
                }
            }

            if let Some((dist, food_pos, _)) = nearest_food {
                if dist < chase_range * 1.5 {
                    // Look TOWARDS food - weak strength (casual interest)
                    let look_dir = (food_pos - my_pos).normalize_or_zero();
                    target_gaze = look_dir * 0.3;
                    let chase_dir = (my_pos - food_pos).normalize_or_zero();
                    if rand::gen_range(0.0, 1.0) < 0.012 * dt * 60.0 {
                        self.eject_mass(id, chase_dir, rand::gen_range(0.1, 0.2));
                    }
                    self.gazes[id] = self.gazes[id].lerp(target_gaze, 5.0 * dt);
                    continue;
                }
            }

            // Wander: gaze drifts slightly towards velocity, mostly returns to center
            let vel = self.velocities[id];
            if vel.length_squared() > 1.0 {
                target_gaze = vel.normalize() * 0.15;
            } else {
                target_gaze = Vec2::ZERO;
            }
            self.gazes[id] = self.gazes[id].lerp(target_gaze, 2.0 * dt);

            if rand::gen_range(0.0, 1.0) < 0.005 * dt * 60.0 {
                let angle = rand::gen_range(0.0, std::f32::consts::TAU);
                let dir = vec2(angle.cos(), angle.sin());
                self.eject_mass(id, dir, rand::gen_range(0.1, 0.2));
            }
        }
    }

    pub fn update(&mut self, dt: f32, player_id: Option<EntityId>) {
        self.collisions.clear();
        self.deaths.clear();

        for explosion in &mut self.explosions {
            explosion.timer -= dt;
        }
        self.explosions.retain(|e| e.timer > 0.0);

        for awakening in &mut self.awakenings {
            awakening.timer -= dt;
        }
        self.awakenings.retain(|a| a.timer > 0.0);

        self.run_ai(player_id, dt);
        self.maybe_spawn_food(dt);

        let world_center = Vec2::ZERO;
        let n = self.types.len();

        let friction_factor = FRICTION.powf(dt * 60.0);
        for i in 0..n {
            if self.types[i] == EntityType::None {
                continue;
            }
            self.velocities[i] *= friction_factor;
            self.positions[i] += self.velocities[i] * dt;
        }

        for i in 0..n {
            let t = self.types[i];
            if t != EntityType::Cell && t != EntityType::Mote {
                continue;
            }
            let pos = self.positions[i];
            let distance = pos.length();
            if distance > BOUNDARY_RADIUS - BOUNDARY_MARGIN {
                let repulsion_dir = if distance > 0.0 {
                    (world_center - pos).normalize()
                } else {
                    Vec2::X
                };
                let repulsion_strength =
                    ((distance - (BOUNDARY_RADIUS - BOUNDARY_MARGIN)) / BOUNDARY_MARGIN).min(1.0);
                self.velocities[i] += repulsion_dir * repulsion_strength * BOUNDARY_FORCE * dt;
            }
        }

        for i in 0..n {
            if self.types[i] != EntityType::Cell {
                continue;
            }
            let radius = mass_to_radius(self.masses[i]);
            self.masses[i] -= MASS_DECAY_RATE * radius * dt * 0.1;
        }

        let absorbers: Vec<EntityId> = (0..n)
            .filter(|&id| {
                let t = self.types[id];
                t == EntityType::Cell || t == EntityType::Mote
            })
            .collect();

        let mut absorptions: Vec<(EntityId, EntityId, f32, Vec2)> = Vec::new();
        for i in 0..absorbers.len() {
            for j in (i + 1)..absorbers.len() {
                let a = absorbers[i];
                let b = absorbers[j];

                let pos_a = self.positions[a];
                let pos_b = self.positions[b];
                let mass_a = self.masses[a];
                let mass_b = self.masses[b];

                let delta = pos_a - pos_b;
                let dist = delta.length();
                let radius_a = mass_to_radius(mass_a);
                let radius_b = mass_to_radius(mass_b);

                if dist < radius_a + radius_b && dist > 0.0 {
                    let (larger, smaller, pos_larger, smaller_mass) = if mass_a > mass_b {
                        (a, b, pos_a, mass_b)
                    } else {
                        (b, a, pos_b, mass_a)
                    };
                    let smaller_radius = mass_to_radius(smaller_mass);

                    let overlap = (radius_a + radius_b) - dist;
                    let new_smaller_radius = (smaller_radius - overlap).max(0.0);

                    let new_smaller_mass = if new_smaller_radius > 0.0 {
                        (new_smaller_radius / 0.5).powf(1.0 / 0.6)
                    } else {
                        0.0
                    };

                    let transferred = smaller_mass - new_smaller_mass;

                    let dir = (pos_larger - self.positions[smaller]).normalize_or_zero();

                    let contact_pos = pos_larger
                        - dir * mass_to_radius(if mass_a > mass_b { mass_a } else { mass_b });
                    self.collisions.push(Collision {
                        contact_pos,
                        normal: dir,
                        smaller_radius: new_smaller_radius,
                        larger_color: self.colors[larger],
                    });

                    absorptions.push((larger, smaller, transferred, dir));
                }
            }
        }

        for (larger, smaller, transferred, dir) in absorptions {
            if self.types[smaller] == EntityType::None {
                continue;
            }

            let smaller_mass = self.masses[smaller];
            let actual_transferred = transferred.min(smaller_mass);

            if self.types[smaller] == EntityType::Mote
                && smaller_mass - actual_transferred < MIN_MOTE_MASS
            {
                self.masses[larger] += smaller_mass;
                self.deaths.push((smaller, larger));
                self.remove_entity(smaller);
            } else {
                self.masses[larger] += actual_transferred;
                self.masses[smaller] -= actual_transferred;

                let new_larger_radius = mass_to_radius(self.masses[larger]);
                let new_smaller_radius = mass_to_radius(self.masses[smaller]);
                let new_pos =
                    self.positions[larger] - dir * (new_larger_radius + new_smaller_radius);
                self.positions[smaller] = new_pos;
            }
        }

        let mut food_absorptions: Vec<(EntityId, EntityId)> = Vec::new();
        for &absorber in &absorbers {
            let absorber_pos = self.positions[absorber];
            let absorber_radius = mass_to_radius(self.masses[absorber]);

            for food in 0..n {
                if self.types[food] != EntityType::Food {
                    continue;
                }
                let food_pos = self.positions[food];
                let food_radius = mass_to_radius(self.masses[food]);

                let dist = (absorber_pos - food_pos).length();
                if dist < absorber_radius + food_radius {
                    food_absorptions.push((absorber, food));
                }
            }
        }

        for (absorber, food_id) in food_absorptions {
            if self.types[food_id] != EntityType::None {
                self.masses[absorber] += self.masses[food_id];
                self.remove_entity(food_id);
            }
        }

        for i in 0..n {
            if self.types[i] == EntityType::Cell && self.masses[i] < MIN_CELL_MASS {
                self.types[i] = EntityType::Mote;
            }
        }

        for i in 0..n {
            if self.types[i] == EntityType::Mote && self.masses[i] >= MOTE_SENTIENCE_MASS {
                self.types[i] = EntityType::Cell;
                let new_color = random_cell_color();
                self.colors[i] = new_color;
                self.awakenings.push(Awakening {
                    entity: i,
                    color: new_color,
                    timer: AWAKENING_DURATION,
                });
            }
        }
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_cell() {
        let mut world = World::new();
        let id = world.spawn_cell(vec2(10.0, 20.0), 50.0, RED);

        assert!(world.is_cell(id));
        assert_eq!(world.positions[id], vec2(10.0, 20.0));
        assert_eq!(world.masses[id], 50.0);
        assert_eq!(world.velocities[id], Vec2::ZERO);
    }

    #[test]
    fn test_mass_ejection_momentum_conservation() {
        let mut world = World::new();
        let id = world.spawn_cell(Vec2::ZERO, 100.0, RED);

        let initial_mass = world.masses[id];

        world.eject_mass(id, vec2(1.0, 0.0), 1.0);

        let final_cell_mass = world.masses[id];
        let mote_id = (0..world.types.len())
            .find(|&i| world.types[i] == EntityType::Mote)
            .unwrap();
        let mote_mass = world.masses[mote_id];
        let mote_vel = world.velocities[mote_id];
        let cell_vel = world.velocities[id];

        assert!((final_cell_mass + mote_mass - initial_mass).abs() < 0.001);

        let total_momentum = final_cell_mass * cell_vel + mote_mass * mote_vel;
        assert!(total_momentum.length() < 0.1);

        assert!(cell_vel.x < 0.0);
    }

    #[test]
    fn test_absorption_larger_absorbs_smaller() {
        let mut world = World::new();
        let large = world.spawn_cell(Vec2::ZERO, 100.0, RED);
        let small = world.spawn_cell(vec2(5.0, 0.0), 30.0, BLUE);

        let initial_total = world.masses[large] + world.masses[small];

        world.update(0.1, None);

        let large_mass = if world.is_cell(large) {
            world.masses[large]
        } else {
            0.0
        };
        let small_mass = if world.is_cell(small) {
            world.masses[small]
        } else {
            0.0
        };

        assert!(large_mass > 100.0 - 5.0);
        assert!(small_mass < 30.0);

        let final_total = large_mass + small_mass;
        assert!(final_total <= initial_total);
    }

    #[test]
    fn test_boundary_repulsion() {
        let mut world = World::new();
        let id = world.spawn_cell(vec2(BOUNDARY_RADIUS - 10.0, 0.0), 50.0, RED);

        world.update(0.1, None);

        let vel = world.velocities[id];
        assert!(vel.x < 0.0);
    }

    #[test]
    fn test_mass_decay() {
        let mut world = World::new();
        let id = world.spawn_cell(Vec2::ZERO, 100.0, RED);

        world.update(1.0, None);

        let mass = world.masses[id];
        assert!(mass < 100.0);
    }

    #[test]
    fn test_cell_becomes_mote_below_min_mass() {
        let mut world = World::new();
        let id = world.spawn_cell(Vec2::ZERO, MIN_CELL_MASS + 1.0, RED);

        for _ in 0..100 {
            world.update(0.5, None);
        }

        assert!(!world.is_cell(id));
        assert_eq!(world.types[id], EntityType::Mote);
    }

    #[test]
    fn test_food_absorption() {
        let mut world = World::new();
        let cell = world.spawn_cell(Vec2::ZERO, 50.0, RED);
        world.spawn_food(vec2(3.0, 0.0));

        let initial_mass = world.masses[cell];

        world.update(0.1, None);

        let final_mass = world.masses[cell];
        assert!(final_mass > initial_mass - 1.0);
        assert!(world.count_by_type(EntityType::Food) == 0);
    }
}
