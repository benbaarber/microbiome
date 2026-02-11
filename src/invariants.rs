// Physics constants
pub const EJECT_MASS_RATIO: f32 = 0.25;
pub const EJECT_SPEED: f32 = 400.0;
pub const FRICTION: f32 = 0.995;
pub const MIN_CELL_MASS: f32 = 5.0; // Cells below this become motes
pub const MIN_MOTE_MASS: f32 = 0.5; // Motes below this are removed
pub const MOTE_SENTIENCE_MASS: f32 = 500.0; // Motes above this become cells
pub const BOUNDARY_RADIUS: f32 = 500.0;
pub const BOUNDARY_MARGIN: f32 = 50.0;
pub const BOUNDARY_FORCE: f32 = 200.0;
pub const MASS_DECAY_RATE: f32 = 0.5;
pub const FOOD_SPAWN_RATE: f32 = 2.0;
pub const FOOD_MASS: f32 = 10.0;
pub const MAX_FOOD: usize = 50;

// Heat
pub const BASE_CELL_HEAT: f32 = 0.2;
pub const EJECT_HEAT_BOOST: f32 = 0.1;
pub const HEAT_COOL_RATE: f32 = 0.3;
pub const OVERHEAT_RECOVERY: f32 = 0.4;

// Effect durations
pub const EXPLOSION_DURATION: f32 = 0.3;
pub const AWAKENING_DURATION: f32 = 0.5;

pub fn mass_to_radius(mass: f32) -> f32 {
    mass.powf(0.6) * 0.5
}
