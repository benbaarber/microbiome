/// Size of the biome
pub const SIZE: f64 = 500.0;

/// Speed of cells
pub const SPEED: f64 = 2.0;

/// Max frame rate to send through pub socket
pub const FPS: u64 = 30;

/// Frame duration
pub const FRAME_DURATION: u64 = 1000 / FPS;

/// Number of food cells to start with
pub const INITIAL_FOOD_SUPPLY: usize = 20;

/// Interval at which food is spawned, in frames
pub const FOOD_SPAWN_RATE: u64 = FPS;
