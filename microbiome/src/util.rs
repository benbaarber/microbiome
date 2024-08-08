use nalgebra as na;
use rand::{distributions::uniform::SampleRange, Rng};
use random_color::RandomColor;

use crate::{constants::SIZE, P2};

/// Convert mass to radius using a simplified version of the volume formula
pub fn mass_to_radius(mass: f64) -> f64 {
    mass.powf(1.0 / 3.0)
}

/// Generate a random position and mass from a given range
///
/// **Returns** (pos, mass, radius, color)
pub fn random_cell(mass_range: impl SampleRange<i32>) -> (P2, f64, f64, String) {
    let mut rng = rand::thread_rng();
    let mass = rng.gen_range(mass_range) as f64;
    let pos = na::point![
        rng.gen::<f64>() * SIZE as f64,
        rng.gen::<f64>() * SIZE as f64
    ];
    let radius = mass_to_radius(mass);
    let color = random_color();
    (pos, mass, radius, color)
}

/// Generate a random hex color
pub fn random_color() -> String {
    RandomColor::new().to_hex()
}
