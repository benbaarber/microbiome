use clap::Parser;
use macroquad::prelude::*;

use crate::brain;
use crate::invariants;
use crate::rendering::Renderer;
use crate::simulation::{EntityId, EntityType, World, WorldConfig};

const CHARGE_TIME: f32 = 0.5;
const MIN_CHARGE: f32 = 0.1;

#[derive(Clone, Copy, clap::ValueEnum)]
pub enum Controller {
    Simple,
    Mlp,
}

#[derive(Parser)]
pub struct Cli {
    /// Simulation mode
    #[arg(default_value = "simple")]
    pub controller: Controller,
}

#[derive(Clone, Copy)]
enum GameMode {
    View,
    Control(EntityId),
    Spectate(EntityId),
}

pub struct Game {
    world: World,
    renderer: Renderer,
    config: WorldConfig,
    state: GameMode,
    charge: f32,
    last_mouse_pos: Option<Vec2>,
    time_scale: f32,
    mode: Controller,
}

impl Game {
    pub fn start(args: &Cli) -> Self {
        let config = WorldConfig::default();
        let world = World::from_config(&config);

        Self {
            world,
            renderer: Renderer::new(),
            config,
            state: GameMode::View,
            charge: 0.0,
            last_mouse_pos: None,
            time_scale: 1.0,
            mode: args.controller,
        }
    }

    fn active_player(&self) -> Option<EntityId> {
        match self.state {
            GameMode::Control(id) => Some(id),
            _ => None,
        }
    }

    /// Returns true if the game should quit.
    pub fn update(&mut self) -> bool {
        let dt = get_frame_time().min(0.05) * self.time_scale;

        if self.handle_input(dt) {
            return true;
        }

        let player_id = self.active_player();
        let think_fn = match self.mode {
            Controller::Simple => brain::simple::think,
            Controller::Mlp => todo!(),
        };
        let n = self.world.types.len();
        for id in 0..n {
            if Some(id) == player_id || self.world.types[id] != EntityType::Cell {
                continue;
            }
            let action = think_fn(&self.world, id, dt);
            if let Some((dir, amount)) = action.eject {
                self.world.eject_mass(id, dir, amount);
            }
            self.world.gazes[id] = self.world.gazes[id].lerp(action.gaze, action.gaze_speed * dt);
        }
        self.world.update(dt);

        clear_background(Color::new(0.02, 0.02, 0.06, 1.0));
        let time = get_time() as f32;
        self.renderer.render(&self.world, time);
        self.draw_hud();

        false
    }

    fn handle_input(&mut self, dt: f32) -> bool {
        if is_key_pressed(KeyCode::Q) {
            return true;
        }

        if is_key_pressed(KeyCode::Minus) {
            self.time_scale = (self.time_scale / 2.0).max(0.25);
        }
        if is_key_pressed(KeyCode::Equal) {
            self.time_scale = (self.time_scale * 2.0).min(4.0);
        }

        if is_key_pressed(KeyCode::R) {
            let world = World::from_config(&self.config);
            self.world = world;
            self.state = GameMode::View;
            self.renderer.camera_pos = Vec2::ZERO;
            self.renderer.zoom = 1.0;
            self.charge = 0.0;
        }

        let scroll = mouse_wheel().1;
        if scroll != 0.0 {
            self.renderer.zoom *= 1.0 + scroll * 0.005;
            self.renderer.zoom = self.renderer.zoom.clamp(0.2, 3.0);
        }

        // Shift+Left Click to explode any entity
        if is_mouse_button_pressed(MouseButton::Left) && shift_held() {
            let mouse = mouse_position();
            let world_mouse = self.renderer.screen_to_world(Vec2::from(mouse));
            if let Some(target) = self.world.entity_at_point(world_mouse, |t| {
                t == EntityType::Cell || t == EntityType::Mote
            }) {
                let was_player = matches!(self.state, GameMode::Control(id) if target == id);
                self.world.explode(target);
                if was_player {
                    self.state = GameMode::View;
                }
            }
        }

        // Ctrl+Left Click to take over control of a cell
        if is_mouse_button_pressed(MouseButton::Left) && ctrl_held() {
            let mouse = mouse_position();
            let world_mouse = self.renderer.screen_to_world(vec2(mouse.0, mouse.1));
            if let Some(target) = self
                .world
                .entity_at_point(world_mouse, |t| t == EntityType::Cell)
            {
                self.state = GameMode::Control(target);
                self.charge = 0.0;
            }
        }

        match self.state {
            GameMode::View => {
                self.charge = 0.0;
                let mouse = mouse_position();
                let mouse_pos = vec2(mouse.0, mouse.1);

                if is_mouse_button_pressed(MouseButton::Left) && !modifier_held() {
                    let world_mouse = self.renderer.screen_to_world(mouse_pos);
                    if let Some(target) = self
                        .world
                        .entity_at_point(world_mouse, |t| t == EntityType::Cell)
                    {
                        self.state = GameMode::Spectate(target);
                    }
                }

                if is_mouse_button_down(MouseButton::Right) {
                    if let Some(last) = self.last_mouse_pos {
                        let delta = (last - mouse_pos) / self.renderer.zoom;
                        self.renderer.camera_pos += delta;
                    }
                    self.last_mouse_pos = Some(mouse_pos);
                } else {
                    self.last_mouse_pos = None;
                }
            }
            GameMode::Control(target) => {
                self.renderer.camera_pos = self
                    .renderer
                    .camera_pos
                    .lerp(self.world.positions[target], 5.0 * dt);

                let player_pos = self.world.positions[target];
                let player_radius = invariants::mass_to_radius(self.world.masses[target]);
                let mouse = mouse_position();
                let world_mouse = self.renderer.screen_to_world(Vec2::from(mouse));
                let to_cursor = world_mouse - player_pos;
                let cursor_dist = to_cursor.length();
                let look_dir = to_cursor.normalize_or_zero();
                let gaze_strength = (cursor_dist / player_radius).min(1.0);
                let target_gaze = look_dir * gaze_strength;
                self.world.gazes[target] = self.world.gazes[target].lerp(target_gaze, 8.0 * dt);

                if is_mouse_button_down(MouseButton::Left) && !modifier_held() {
                    self.charge = (self.charge + dt / CHARGE_TIME).min(1.0);
                }
                if is_mouse_button_released(MouseButton::Left) && !modifier_held() {
                    let eject_dir = world_mouse - player_pos;
                    if eject_dir.length_squared() > 0.0 {
                        let amount = MIN_CHARGE + self.charge * (1.0 - MIN_CHARGE);
                        self.world.eject_mass(target, eject_dir.normalize(), amount);
                    }
                    self.charge = 0.0;
                }
                if is_key_pressed(KeyCode::Escape) || is_mouse_button_pressed(MouseButton::Right) {
                    self.state = GameMode::View;
                }
                if !self.world.is_cell(target) {
                    self.state = GameMode::View;
                }
            }
            GameMode::Spectate(target) => {
                if let Some(&(_, killer)) = self
                    .world
                    .deaths
                    .iter()
                    .find(|&&(victim, _)| victim == target)
                {
                    self.state = GameMode::Spectate(killer);
                }

                let current_target = if let GameMode::Spectate(t) = self.state {
                    t
                } else {
                    target
                };

                if current_target < self.world.positions.len()
                    && self.world.types[current_target] != EntityType::None
                {
                    self.renderer.camera_pos = self
                        .renderer
                        .camera_pos
                        .lerp(self.world.positions[current_target], 5.0 * dt);
                } else {
                    self.state = GameMode::View;
                }

                if is_mouse_button_pressed(MouseButton::Left) && !modifier_held() {
                    let mouse = mouse_position();
                    let world_mouse = self.renderer.screen_to_world(vec2(mouse.0, mouse.1));
                    if let Some(new_target) = self
                        .world
                        .entity_at_point(world_mouse, |t| t == EntityType::Cell)
                    {
                        self.state = GameMode::Spectate(new_target);
                    }
                }

                if is_key_pressed(KeyCode::Escape) || is_mouse_button_pressed(MouseButton::Right) {
                    self.state = GameMode::View;
                }
            }
        }

        false
    }

    fn draw_entity_stats(&self, target: EntityId, x: f32) {
        if target >= self.world.types.len() {
            return;
        }
        let label = format!(
            "(#{})   Mass: {:>5.0}   Heat: ",
            target, self.world.masses[target]
        );
        draw_text(&label, x, 30.0, 24.0, WHITE);

        let heat_ratio = self.world.heats[target].clamp(0.0, 1.0);
        let filled = if heat_ratio > 0.0 {
            (heat_ratio * 10.0).ceil() as usize
        } else {
            0
        };
        let mut bar_x = x + measure_text(&label, None, 24, 1.0).width;
        let block_w = measure_text("\u{2588}", None, 24, 1.0).width;

        draw_text("[", bar_x, 30.0, 24.0, GRAY);
        bar_x += measure_text("[", None, 24, 1.0).width;

        for i in 0..10 {
            let t = i as f32 / 9.0;
            let r = t.powf(0.6);
            let g = (1.0 - (2.0 * t - 1.0).powi(2)) * 0.3;
            let b = (1.0 - t).powf(0.6);
            if i < filled {
                draw_text(
                    "\u{2588}",
                    bar_x + i as f32 * block_w,
                    30.0,
                    24.0,
                    Color::new(r, g, b, 1.0),
                );
            } else {
                draw_text(
                    "\u{2588}",
                    bar_x + i as f32 * block_w,
                    30.0,
                    24.0,
                    Color::new(r, g, b, 0.15),
                );
            }
        }

        let after_bar = bar_x + 10.0 * block_w;
        draw_text("]", after_bar, 30.0, 24.0, GRAY);
        let pct_x = after_bar + measure_text("] ", None, 24, 1.0).width;
        draw_text(
            &format!("{:.0}%", heat_ratio * 100.0),
            pct_x,
            30.0,
            24.0,
            WHITE,
        );
    }

    fn draw_hud(&self) {
        match self.state {
            GameMode::View => {
                draw_text("VIEW", 10.0, 30.0, 24.0, LIGHTGRAY);
            }
            GameMode::Control(target) => {
                draw_text("CONTROL", 10.0, 30.0, 24.0, GREEN);
                self.draw_entity_stats(target, 110.0);

                let bar_w = 150.0;
                let bar_h = 16.0;
                let bar_x = (screen_width() - bar_w) / 2.0;
                let bar_y = 15.0;
                draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(0.2, 0.2, 0.2, 0.8));
                let fill_color = if self.charge < 0.5 {
                    Color::new(0.3, 0.7, 1.0, 1.0)
                } else if self.charge < 0.9 {
                    Color::new(1.0, 0.8, 0.2, 1.0)
                } else {
                    Color::new(1.0, 0.3, 0.2, 1.0)
                };
                draw_rectangle(
                    bar_x + 2.0,
                    bar_y + 2.0,
                    (bar_w - 4.0) * self.charge,
                    bar_h - 4.0,
                    fill_color,
                );
                draw_rectangle_lines(bar_x, bar_y, bar_w, bar_h, 2.0, WHITE);
            }
            GameMode::Spectate(target) => {
                draw_text("SPECTATE", 10.0, 30.0, 24.0, YELLOW);
                self.draw_entity_stats(target, 120.0);
            }
        }
        draw_text(
            &format!(
                "Cells: {} | Motes: {} | Food: {}",
                self.world.count_by_type(EntityType::Cell),
                self.world.count_by_type(EntityType::Mote),
                self.world.count_by_type(EntityType::Food)
            ),
            10.0,
            55.0,
            18.0,
            GRAY,
        );
        let speed_text = if self.time_scale == 1.0 {
            "1x".to_string()
        } else if self.time_scale < 1.0 {
            format!("{:.2}x", self.time_scale)
        } else {
            format!("{}x", self.time_scale as i32)
        };
        draw_text(
            &format!("Speed: {} (-/=)", speed_text),
            screen_width() - 130.0,
            30.0,
            18.0,
            GRAY,
        );
        draw_text(
            "Shift+Click: Explode | Ctrl+Click: Control | Scroll: Zoom | R: Reset | Q: Quit",
            10.0,
            80.0,
            16.0,
            DARKGRAY,
        );
        let mode_controls = match self.state {
            GameMode::View => "Click: Spectate | Right-drag: Pan",
            GameMode::Control(_) => "Hold+Release: Eject mass | Right-click/Esc: Release",
            GameMode::Spectate(_) => "Click: Switch | Right-click/Esc: Stop spectating",
        };
        draw_text(mode_controls, 10.0, 98.0, 16.0, DARKGRAY);
    }
}

fn shift_held() -> bool {
    is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift)
}

fn ctrl_held() -> bool {
    is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)
}

fn modifier_held() -> bool {
    shift_held() || ctrl_held()
}
