/// Size of the biome
pub const SIZE: f64 = 500.0;

/// Base speed of cells, decreases with mass
pub const BASE_SPEED: f64 = 4.0;

/// Max frame rate to send through pub socket
pub const FPS: u64 = 30;

/// Frame duration
pub const FRAME_DURATION: u64 = 1000 / FPS;

/// Number of NPCs spawned initially
pub const INITIAL_NUM_NPCS: usize = 10;

/// Number of food cells to start with
pub const INITIAL_FOOD_SUPPLY: usize = 20;

/// How much food is spawned per second
pub const FOOD_SPAWN_RATE: u64 = 3;

/// Base mass decay rate, increases with mass
pub const BASE_MASS_DECAY_RATE: f64 = 1.0;

/// Calculate radius from mass
pub fn radius(mass: f64) -> f64 {
    mass.sqrt()
}

/// Calculate speed from mass
pub fn speed(mass: f64) -> f64 {
    BASE_SPEED / ((mass / 50.0).sqrt() + 1.0)
}

/// Calculate mass decay from mass
pub fn mass_decay(mass: f64) -> f64 {
    todo!()
}
