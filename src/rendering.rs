use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;

use crate::invariants::*;
use crate::simulation::{Awakening, Collision, EntityId, EntityType, Explosion, World};

pub struct Renderer {
    material: Material,
    pub camera_pos: Vec2,
    pub zoom: f32,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            material: load_glow_material(),
            camera_pos: Vec2::ZERO,
            zoom: 1.0,
        }
    }

    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let screen_center = vec2(screen_width() / 2.0, screen_height() / 2.0);
        (screen_pos - screen_center) / self.zoom + self.camera_pos
    }

    fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let screen_center = vec2(screen_width() / 2.0, screen_height() / 2.0);
        screen_center + (world_pos - self.camera_pos) * self.zoom
    }

    pub fn render(&self, world: &World, player_id: EntityId, time: f32) {
        let zoom = self.zoom;
        let boundary_screen = self.world_to_screen(Vec2::ZERO);
        let boundary_color = Color::new(0.3, 0.2, 0.5, 0.3);
        for i in 0..3 {
            let alpha = 0.15 - i as f32 * 0.04;
            let offset = i as f32 * 8.0;
            draw_circle_lines(
                boundary_screen.x,
                boundary_screen.y,
                BOUNDARY_RADIUS * zoom + offset,
                3.0 - i as f32,
                Color::new(boundary_color.r, boundary_color.g, boundary_color.b, alpha),
            );
        }

        let n = world.types.len();

        for i in 0..n {
            if world.types[i] != EntityType::Food {
                continue;
            }
            let screen_pos = self.world_to_screen(world.positions[i]);
            let radius = mass_to_radius(world.masses[i]) * zoom;
            let food_color = Color::new(0.3, 0.8, 0.4, 0.8);
            let seed = i as f32 * 7.31;
            draw_glowing_circle(
                &self.material,
                screen_pos.x,
                screen_pos.y,
                radius,
                food_color,
                0.3,
                seed,
                time,
                Vec2::ZERO,
            );
        }

        for i in 0..n {
            if world.types[i] != EntityType::Mote {
                continue;
            }
            let screen_pos = self.world_to_screen(world.positions[i]);
            let radius = mass_to_radius(world.masses[i]) * zoom;
            let mote_color = Color::new(
                world.colors[i].r * 0.8,
                world.colors[i].g * 0.8,
                world.colors[i].b * 0.8,
                0.7,
            );
            let seed = i as f32 * 7.31;
            draw_glowing_circle(
                &self.material,
                screen_pos.x,
                screen_pos.y,
                radius.max(3.0),
                mote_color,
                0.35,
                seed,
                time,
                Vec2::ZERO,
            );
        }

        for i in 0..n {
            if world.types[i] != EntityType::Cell {
                continue;
            }
            let screen_pos = self.world_to_screen(world.positions[i]);
            let radius = mass_to_radius(world.masses[i]) * zoom;
            let is_player = i == player_id;
            let glow_intensity = if is_player { 0.5 } else { 0.4 };
            let alpha = if is_player { 0.95 } else { 0.85 };
            let color = Color::new(
                world.colors[i].r,
                world.colors[i].g,
                world.colors[i].b,
                alpha,
            );
            let seed = i as f32 * 7.31;
            let gaze = world.gazes[i];

            draw_glowing_circle(
                &self.material,
                screen_pos.x,
                screen_pos.y,
                radius,
                color,
                glow_intensity,
                seed,
                time,
                gaze,
            );
        }

        for collision in &world.collisions {
            let screen_pos = self.world_to_screen(collision.contact_pos);
            draw_collision(collision, screen_pos, zoom);
        }

        for explosion in &world.explosions {
            let screen_pos = self.world_to_screen(explosion.pos);
            draw_explosion(explosion, screen_pos, zoom);
        }

        for awakening in &world.awakenings {
            let entity = awakening.entity;
            if entity < world.positions.len() {
                let screen_pos = self.world_to_screen(world.positions[entity]);
                let radius = mass_to_radius(world.masses[entity]);
                draw_awakening(awakening, screen_pos, radius, zoom);
            }
        }
    }
}

fn load_glow_material() -> Material {
    load_material(
        ShaderSource::Glsl {
            vertex: include_str!("shaders/glow.vert"),
            fragment: include_str!("shaders/glow.frag"),
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("color", UniformType::Float4),
                UniformDesc::new("glow_intensity", UniformType::Float1),
                UniformDesc::new("time", UniformType::Float1),
                UniformDesc::new("seed", UniformType::Float1),
                UniformDesc::new("gaze_offset", UniformType::Float2),
            ],
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                alpha_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::One,
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("Failed to load glow shader")
}

fn draw_explosion(explosion: &Explosion, screen_pos: Vec2, zoom: f32) {
    let progress = 1.0 - (explosion.timer / EXPLOSION_DURATION);
    let base_radius = explosion.radius * zoom;

    // Expanding ring
    let ring_radius = base_radius * (0.5 + progress * 2.0);
    let ring_alpha = (1.0 - progress) * 0.8;

    // Hot white core that shrinks
    let core_radius = base_radius * (1.0 - progress * 0.8);
    let core_alpha = (1.0 - progress).powf(0.5);

    // Draw outer glow rings
    for i in (0..6).rev() {
        let t = i as f32 / 5.0;
        let r = ring_radius * (1.0 + t * 0.5);
        let a = ring_alpha * (1.0 - t) * 0.4;
        let glow_color = Color::new(explosion.color.r, explosion.color.g, explosion.color.b, a);
        draw_circle(screen_pos.x, screen_pos.y, r, glow_color);
    }

    // Draw hot core layers (white to color gradient)
    for i in (0..5).rev() {
        let t = i as f32 / 4.0;
        let r = core_radius * (0.3 + t * 0.7);
        let a = core_alpha * (1.0 - t * 0.3);
        // Blend from white at center to color at edge
        let blend = t;
        let cr = 1.0 * (1.0 - blend) + explosion.color.r * blend;
        let cg = 1.0 * (1.0 - blend) + explosion.color.g * blend;
        let cb = 1.0 * (1.0 - blend) + explosion.color.b * blend;
        let core_color = Color::new(cr, cg, cb, a);
        draw_circle(screen_pos.x, screen_pos.y, r, core_color);
    }
}

fn draw_awakening(awakening: &Awakening, screen_pos: Vec2, radius: f32, zoom: f32) {
    let progress = 1.0 - (awakening.timer / AWAKENING_DURATION);
    let base_radius = radius * zoom;

    // Intensity curve: fade in, peak at 0.3, fade out
    let intensity = if progress < 0.3 {
        progress / 0.3
    } else {
        1.0 - ((progress - 0.3) / 0.7)
    };
    let intensity = intensity.max(0.0);

    // White amount decreases as progress increases
    let white_blend = (1.0 - progress).powf(1.5);

    // Core white glow (shrinks over time)
    let core_size = 1.0 - progress * 0.7;
    for i in 0..8 {
        let t = i as f32 / 7.0;
        let r = base_radius * core_size * (1.0 - t * 0.8);
        let a = intensity * white_blend * (1.0 - t) * 0.5;
        if a > 0.01 {
            draw_circle(screen_pos.x, screen_pos.y, r, Color::new(1.0, 1.0, 1.0, a));
        }
    }

    // Color glow that expands outward
    let color_intensity = intensity * (1.0 - white_blend * 0.5);
    for i in 0..6 {
        let t = i as f32 / 5.0;
        let expand = progress * 0.4;
        let r = base_radius * (0.8 + t * 0.5 + expand);
        let a = color_intensity * (1.0 - t) * 0.35;
        if a > 0.01 {
            draw_circle(
                screen_pos.x,
                screen_pos.y,
                r,
                Color::new(awakening.color.r, awakening.color.g, awakening.color.b, a),
            );
        }
    }
}

fn draw_collision(collision: &Collision, screen_pos: Vec2, zoom: f32) {
    let base_size = (collision.smaller_radius * zoom * 0.8).max(6.0);
    let tangent = vec2(-collision.normal.y, collision.normal.x);
    let segments = 10;

    // Outer glow layers (colored, fading out)
    for i in (0..5).rev() {
        let t = i as f32 / 4.0;
        let alpha = 0.3 * (1.0 - t);
        let scale = 1.0 + t * 1.2;

        let width = base_size * 0.8 * scale;
        let height = base_size * 0.25 * scale;

        let color = Color::new(
            collision.larger_color.r,
            collision.larger_color.g,
            collision.larger_color.b,
            alpha,
        );

        for j in 0..segments {
            let a1 = (j as f32 / segments as f32) * std::f32::consts::TAU;
            let a2 = ((j + 1) as f32 / segments as f32) * std::f32::consts::TAU;
            let p1 =
                screen_pos + tangent * (a1.cos() * width) + collision.normal * (a1.sin() * height);
            let p2 =
                screen_pos + tangent * (a2.cos() * width) + collision.normal * (a2.sin() * height);
            draw_triangle(screen_pos, p1, p2, color);
        }
    }

    // Bright core (white-ish center)
    for i in (0..3).rev() {
        let t = i as f32 / 2.0;
        let alpha = 0.9 * (1.0 - t * 0.5);
        let scale = 0.3 + t * 0.4;

        let width = base_size * 0.8 * scale;
        let height = base_size * 0.25 * scale;

        // Lerp from white to bright color
        let bright_r = (collision.larger_color.r * 0.5 + 0.5).min(1.0);
        let bright_g = (collision.larger_color.g * 0.5 + 0.5).min(1.0);
        let bright_b = (collision.larger_color.b * 0.5 + 0.5).min(1.0);
        let color = Color::new(bright_r, bright_g, bright_b, alpha);

        for j in 0..segments {
            let a1 = (j as f32 / segments as f32) * std::f32::consts::TAU;
            let a2 = ((j + 1) as f32 / segments as f32) * std::f32::consts::TAU;
            let p1 =
                screen_pos + tangent * (a1.cos() * width) + collision.normal * (a1.sin() * height);
            let p2 =
                screen_pos + tangent * (a2.cos() * width) + collision.normal * (a2.sin() * height);
            draw_triangle(screen_pos, p1, p2, color);
        }
    }
}

fn draw_glowing_circle(
    material: &Material,
    x: f32,
    y: f32,
    radius: f32,
    color: Color,
    glow_intensity: f32,
    seed: f32,
    time: f32,
    gaze: Vec2,
) {
    let glow_size = radius * 1.1;

    gl_use_material(material);
    material.set_uniform("color", color.to_vec());
    material.set_uniform("glow_intensity", glow_intensity);
    material.set_uniform("time", time);
    material.set_uniform("seed", seed);
    material.set_uniform("gaze_offset", gaze);

    draw_rectangle(
        x - glow_size,
        y - glow_size,
        glow_size * 2.0,
        glow_size * 2.0,
        WHITE,
    );

    gl_use_default_material();
}
