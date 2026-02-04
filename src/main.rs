use macroquad::prelude::*;

use microbiome::invariants;
use microbiome::rendering::Renderer;
use microbiome::simulation::{EntityId, EntityType, World, WorldConfig};

fn shift_held() -> bool {
    is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift)
}

fn ctrl_held() -> bool {
    is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)
}

fn modifier_held() -> bool {
    shift_held() || ctrl_held()
}

#[derive(Clone, Copy, PartialEq)]
enum PlayerState {
    Alive,
    Dead,
    Spectate(EntityId),
}

#[macroquad::main("Microbiome")]
async fn main() {
    let mut renderer = Renderer::new();

    let config = WorldConfig::default();
    let (mut world, mut player) = World::from_config(&config);
    let mut charge = 0.0f32;
    let mut state = PlayerState::Alive;
    let mut last_mouse_pos: Option<Vec2> = None;
    let mut time_scale = 1.0f32;
    const CHARGE_TIME: f32 = 0.5;
    const MIN_CHARGE: f32 = 0.1;

    loop {
        if is_key_pressed(KeyCode::Minus) {
            time_scale = (time_scale / 2.0).max(0.25);
        }
        if is_key_pressed(KeyCode::Equal) {
            time_scale = (time_scale * 2.0).min(4.0);
        }

        let dt = get_frame_time().min(0.05) * time_scale;

        if is_key_pressed(KeyCode::Q) {
            break;
        }

        if is_key_pressed(KeyCode::R) {
            let (new_world, new_player) = World::from_config(&config);
            world = new_world;
            player = new_player;
            state = PlayerState::Alive;
            renderer.camera_pos = Vec2::ZERO;
            renderer.zoom = 1.0;
            charge = 0.0;
        }

        let scroll = mouse_wheel().1;
        if scroll != 0.0 {
            renderer.zoom *= 1.0 + scroll * 0.005;
            renderer.zoom = renderer.zoom.clamp(0.2, 3.0);
        }

        // Shift+Left Click to explode any entity (works from all modes)
        if is_mouse_button_pressed(MouseButton::Left) && shift_held() {
            let mouse = mouse_position();
            let world_mouse = renderer.screen_to_world(vec2(mouse.0, mouse.1));
            if let Some(target) = world.entity_at_point(world_mouse, |t| {
                t == EntityType::Cell || t == EntityType::Mote
            }) {
                let was_player = state == PlayerState::Alive && target == player;
                world.explode(target);
                if was_player {
                    state = PlayerState::Dead;
                }
            }
        }

        // Ctrl+Left Click to take over control of a cell (works from all modes)
        if is_mouse_button_pressed(MouseButton::Left) && ctrl_held() {
            let mouse = mouse_position();
            let world_mouse = renderer.screen_to_world(vec2(mouse.0, mouse.1));
            if let Some(target) = world.entity_at_point(world_mouse, |t| t == EntityType::Cell) {
                player = target;
                state = PlayerState::Alive;
                charge = 0.0;
            }
        }

        match state {
            PlayerState::Alive => {
                renderer.camera_pos = renderer.camera_pos.lerp(world.positions[player], 5.0 * dt);

                // Player looks towards mouse cursor, strength based on distance
                let player_pos = world.positions[player];
                let player_radius = invariants::mass_to_radius(world.masses[player]);
                let mouse = mouse_position();
                let world_mouse = renderer.screen_to_world(vec2(mouse.0, mouse.1));
                let to_cursor = world_mouse - player_pos;
                let cursor_dist = to_cursor.length();
                let look_dir = to_cursor.normalize_or_zero();
                // Gaze strength scales with distance: 0 at center, 1.0 at edge and beyond
                let gaze_strength = (cursor_dist / player_radius).min(1.0);
                let target_gaze = look_dir * gaze_strength;
                world.gazes[player] = world.gazes[player].lerp(target_gaze, 8.0 * dt);

                if is_mouse_button_down(MouseButton::Left) && !modifier_held() {
                    charge = (charge + dt / CHARGE_TIME).min(1.0);
                }
                if is_mouse_button_released(MouseButton::Left) && !modifier_held() {
                    let eject_dir = world_mouse - player_pos;
                    if eject_dir.length_squared() > 0.0 {
                        let amount = MIN_CHARGE + charge * (1.0 - MIN_CHARGE);
                        world.eject_mass(player, eject_dir.normalize(), amount);
                    }
                    charge = 0.0;
                }
                if !world.is_cell(player) {
                    state = PlayerState::Dead;
                }
            }
            PlayerState::Dead => {
                charge = 0.0;
                let mouse = mouse_position();
                let mouse_pos = vec2(mouse.0, mouse.1);

                // Left-click to spectate a cell (not with modifiers)
                if is_mouse_button_pressed(MouseButton::Left) && !modifier_held() {
                    let world_mouse = renderer.screen_to_world(mouse_pos);
                    if let Some(target) =
                        world.entity_at_point(world_mouse, |t| t == EntityType::Cell)
                    {
                        state = PlayerState::Spectate(target);
                    }
                }

                // Right-drag to pan
                if is_mouse_button_down(MouseButton::Right) {
                    if let Some(last) = last_mouse_pos {
                        let delta = (last - mouse_pos) / renderer.zoom;
                        renderer.camera_pos += delta;
                    }
                    last_mouse_pos = Some(mouse_pos);
                } else {
                    last_mouse_pos = None;
                }
            }
            PlayerState::Spectate(target) => {
                // Check if spectated entity was consumed - if so, follow the killer
                if let Some(&(_, killer)) =
                    world.deaths.iter().find(|&&(victim, _)| victim == target)
                {
                    state = PlayerState::Spectate(killer);
                }

                // Extract current target (may have changed above)
                let current_target = if let PlayerState::Spectate(t) = state {
                    t
                } else {
                    target
                };

                if current_target < world.positions.len()
                    && world.types[current_target] != EntityType::None
                {
                    renderer.camera_pos =
                        renderer.camera_pos.lerp(world.positions[current_target], 5.0 * dt);
                } else {
                    state = PlayerState::Dead;
                }

                // Left-click to spectate a different cell (not with modifiers)
                if is_mouse_button_pressed(MouseButton::Left) && !modifier_held() {
                    let mouse = mouse_position();
                    let world_mouse = renderer.screen_to_world(vec2(mouse.0, mouse.1));
                    if let Some(new_target) =
                        world.entity_at_point(world_mouse, |t| t == EntityType::Cell)
                    {
                        state = PlayerState::Spectate(new_target);
                    }
                }

                if is_key_pressed(KeyCode::Escape) || is_mouse_button_pressed(MouseButton::Right) {
                    state = PlayerState::Dead;
                }
            }
        }

        let active_player = if state == PlayerState::Alive {
            Some(player)
        } else {
            None
        };
        world.update(dt, active_player);

        clear_background(Color::new(0.02, 0.02, 0.06, 1.0));
        let time = get_time() as f32;
        renderer.render(&world, player, time);

        match state {
            PlayerState::Alive => {
                draw_text("ALIVE", 10.0, 30.0, 24.0, GREEN);
                draw_text(
                    &format!("(#{})   Mass: {:.0}", player, world.masses[player]),
                    80.0,
                    30.0,
                    24.0,
                    WHITE,
                );

                let bar_w = 150.0;
                let bar_h = 16.0;
                let bar_x = (screen_width() - bar_w) / 2.0;
                let bar_y = 15.0;
                draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(0.2, 0.2, 0.2, 0.8));
                let fill_color = if charge < 0.5 {
                    Color::new(0.3, 0.7, 1.0, 1.0)
                } else if charge < 0.9 {
                    Color::new(1.0, 0.8, 0.2, 1.0)
                } else {
                    Color::new(1.0, 0.3, 0.2, 1.0)
                };
                draw_rectangle(
                    bar_x + 2.0,
                    bar_y + 2.0,
                    (bar_w - 4.0) * charge,
                    bar_h - 4.0,
                    fill_color,
                );
                draw_rectangle_lines(bar_x, bar_y, bar_w, bar_h, 2.0, WHITE);
            }
            PlayerState::Dead => {
                draw_text("DEAD", 10.0, 30.0, 24.0, RED);
            }
            PlayerState::Spectate(target) => {
                draw_text("SPECTATE", 10.0, 30.0, 24.0, YELLOW);
                if target < world.masses.len() {
                    draw_text(
                        &format!("(#{})   Mass: {:.0}", target, world.masses[target]),
                        120.0,
                        30.0,
                        24.0,
                        WHITE,
                    );
                }
            }
        }
        draw_text(
            &format!(
                "Cells: {} | Motes: {} | Food: {}",
                world.count_by_type(EntityType::Cell),
                world.count_by_type(EntityType::Mote),
                world.count_by_type(EntityType::Food)
            ),
            10.0,
            55.0,
            18.0,
            GRAY,
        );
        let speed_text = if time_scale == 1.0 {
            "1x".to_string()
        } else if time_scale < 1.0 {
            format!("{:.2}x", time_scale)
        } else {
            format!("{}x", time_scale as i32)
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
        let mode_controls = match state {
            PlayerState::Alive => "Hold+Release: Eject mass",
            PlayerState::Dead => "Click: Spectate | Right-drag: Pan",
            PlayerState::Spectate(_) => "Click: Switch | Right-click/Esc: Stop spectating",
        };
        draw_text(mode_controls, 10.0, 98.0, 16.0, DARKGRAY);

        next_frame().await;
    }
}
