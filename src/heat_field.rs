use macroquad::prelude::*;

use crate::invariants::*;

pub struct HeatField {
    pub cells: Vec<f32>,
    scratch: Vec<f32>,
    pub resolution: usize,
    pub world_extent: f32,
}

impl HeatField {
    pub fn new(resolution: usize, world_extent: f32) -> Self {
        let n = resolution * resolution;
        Self {
            cells: vec![0.0; n],
            scratch: vec![0.0; n],
            resolution,
            world_extent,
        }
    }

    fn world_to_grid(&self, pos: Vec2) -> (f32, f32) {
        let cell_size = 2.0 * self.world_extent / self.resolution as f32;
        let gx = (pos.x + self.world_extent) / cell_size;
        let gy = (pos.y + self.world_extent) / cell_size;
        (gx, gy)
    }

    pub fn deposit(&mut self, pos: Vec2, heat: f32, radius: f32) {
        if heat <= 0.0 {
            return;
        }
        let cell_size = 2.0 * self.world_extent / self.resolution as f32;
        let (gx, gy) = self.world_to_grid(pos);
        let grid_radius = (radius / cell_size).max(1.0);
        let r = grid_radius.ceil() as i32;
        let res = self.resolution as i32;

        for dy in -r..=r {
            for dx in -r..=r {
                let ix = gx as i32 + dx;
                let iy = gy as i32 + dy;
                if ix < 0 || iy < 0 || ix >= res || iy >= res {
                    continue;
                }
                let dist = ((dx as f32 - (gx.fract() - 0.5)).powi(2)
                    + (dy as f32 - (gy.fract() - 0.5)).powi(2))
                .sqrt();
                let weight = (1.0 - dist / grid_radius).max(0.0);
                self.cells[iy as usize * self.resolution + ix as usize] +=
                    heat * weight * HEAT_FIELD_DEPOSIT;
            }
        }
    }

    pub fn diffuse_and_decay(&mut self, dt: f32) {
        let res = self.resolution;
        let cell_size = 2.0 * self.world_extent / res as f32;
        let alpha = HEAT_FIELD_DIFFUSION * dt / (cell_size * cell_size);
        let decay = (-HEAT_FIELD_DECAY * dt).exp();

        for y in 0..res {
            for x in 0..res {
                let idx = y * res + x;
                let center = self.cells[idx];

                let left = if x > 0 { self.cells[idx - 1] } else { 0.0 };
                let right = if x < res - 1 {
                    self.cells[idx + 1]
                } else {
                    0.0
                };
                let up = if y > 0 { self.cells[idx - res] } else { 0.0 };
                let down = if y < res - 1 {
                    self.cells[idx + res]
                } else {
                    0.0
                };

                let laplacian = left + right + up + down - 4.0 * center;
                self.scratch[idx] = (center + alpha * laplacian) * decay;
            }
        }

        std::mem::swap(&mut self.cells, &mut self.scratch);
    }

    pub fn sample(&self, pos: Vec2) -> f32 {
        let (gx, gy) = self.world_to_grid(pos);
        let gx = gx - 0.5;
        let gy = gy - 0.5;
        let x0 = (gx.floor() as i32).clamp(0, self.resolution as i32 - 1) as usize;
        let y0 = (gy.floor() as i32).clamp(0, self.resolution as i32 - 1) as usize;
        let x1 = (x0 + 1).min(self.resolution - 1);
        let y1 = (y0 + 1).min(self.resolution - 1);
        let fx = gx - gx.floor();
        let fy = gy - gy.floor();

        let c00 = self.cells[y0 * self.resolution + x0];
        let c10 = self.cells[y0 * self.resolution + x1];
        let c01 = self.cells[y1 * self.resolution + x0];
        let c11 = self.cells[y1 * self.resolution + x1];

        let top = c00 * (1.0 - fx) + c10 * fx;
        let bot = c01 * (1.0 - fx) + c11 * fx;
        top * (1.0 - fy) + bot * fy
    }
}
