use clap::Parser;
use macroquad::prelude::*;

use microbiome::game::{Cli, Game};

#[macroquad::main("Microbiome")]
async fn main() {
    let cli = Cli::parse();
    let mut game = Game::start(&cli);
    loop {
        if game.update() {
            break;
        }
        next_frame().await;
    }
}
