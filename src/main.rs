use clap::Parser;
use macroquad::prelude::*;

use microbiome::game::{Cli, Game};
use microbiome::simulation::WorldConfig;

#[macroquad::main("Microbiome")]
async fn main() {
    let cli = Cli::parse();
    let config = WorldConfig {
        cell_count: 20,
        cell_mass: 5000.0,
        cell_explosion_threshold: Some(10000.0),
        ..Default::default()
    };
    let mut game = Game::start(&cli, config);
    loop {
        if game.update() {
            break;
        }
        next_frame().await;
    }
}
