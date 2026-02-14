use clap::Parser;
use macroquad::prelude::*;

use microbiome::game::{Cli, Game};
use microbiome::simulation::WorldConfig;

#[macroquad::main("Microbiome")]
async fn main() {
    let cli = Cli::parse();
    let config = WorldConfig::default();
    let mut game = Game::start(&cli, config);
    loop {
        if game.update() {
            break;
        }
        next_frame().await;
    }
}
