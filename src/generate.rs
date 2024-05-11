use bevy_ecs::system::Resource;
use cosiest_noisiest::NoiseGenerator;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

#[derive(Resource, Clone)]
pub struct Generator {
    seed: [u8; 32],
    height_generator: cosiest_noisiest::NoiseGenerator<f64>,
    hole_generator: cosiest_noisiest::NoiseGenerator<f64>,
    upgrade_rng: ChaCha20Rng,
}

pub struct NoiseSettings {
    wave_length: f64,
    amplitude: f64,
    octaves: usize,
}
impl NoiseSettings {
    pub fn new(wave_length: impl Into<f64>, amplitude: impl Into<f64>, octaves: usize) -> Self {
        Self {
            wave_length:wave_length.into(),
            amplitude:amplitude.into(),
            octaves,
        }
    }
}

impl Generator {
    pub fn from_u64_seed(
        seed: u64,
        height_noise_settings: NoiseSettings,
        hole_noise_settings: NoiseSettings,
    ) -> Self {
        let rng = ChaCha20Rng::seed_from_u64(seed);
        Self::new(rng, height_noise_settings, hole_noise_settings)
    }
    pub fn from_entropy(
        height_noise_settings: NoiseSettings,
        hole_noise_settings: NoiseSettings,
    ) -> Self {
        let rng = ChaCha20Rng::from_entropy();
        Self::new(rng, height_noise_settings, hole_noise_settings)
    }
    pub fn from_seed(
        seed: [u8; 32],
        height_noise_settings: NoiseSettings,
        hole_noise_settings: NoiseSettings,
    ) -> Self {
        let rng = ChaCha20Rng::from_seed(seed);
        Self::new(rng, height_noise_settings, hole_noise_settings)
    }
    pub fn get_seed(&self) -> [u8; 32] {
        self.seed
    }
    pub fn new(
        rng: ChaCha20Rng,
        height_noise_settings: NoiseSettings,
        hole_noise_settings: NoiseSettings,
    ) -> Self {
        let mut height_rng = rng.clone();
        height_rng.set_stream(1);
        let mut upgrade_rng = rng.clone();
        upgrade_rng.set_stream(2);
        let mut hole_rng = rng.clone();
        hole_rng.set_stream(3);
        Self {
            seed: rng.get_seed(),
            height_generator: NoiseGenerator::from_rng(
                height_rng,
                1. / height_noise_settings.wave_length,
                height_noise_settings.amplitude,
                height_noise_settings.octaves,
            ),
            hole_generator: NoiseGenerator::from_rng(
                hole_rng,
                1. / hole_noise_settings.wave_length,
                hole_noise_settings.amplitude,
                hole_noise_settings.octaves,
            ),
            upgrade_rng,
        }
    }
    pub fn get_height(&mut self, x: usize) -> f64 {
        self.height_generator.sample(x)
    }

    pub fn get_upgrade(&mut self) -> usize {
        self.upgrade_rng.gen()
    }

    pub fn is_hole(&mut self,x:usize)->bool{
        self.hole_generator.sample(x) >= self.hole_generator.amplitude * 0.95
    }
}
