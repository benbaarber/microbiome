use rand::RngExt;
use rand_distr::Distribution;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LogUniform {
    log_min: f32,
    log_max: f32,
}

impl LogUniform {
    pub fn new(min: f32, max: f32) -> Self {
        Self {
            log_min: min.ln(),
            log_max: max.ln(),
        }
    }
}

impl Distribution<f32> for LogUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> f32 {
        rng.random_range(self.log_min..self.log_max).exp()
    }
}
