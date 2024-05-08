use bevy_ecs::system::Resource;
use cosiest_noisiest::NoiseGenerator;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

#[derive(Resource, Clone)]
pub struct Generator {
    seed:[u8;32],
    height_generator: cosiest_noisiest::NoiseGenerator<f64>,
}

impl Generator {
    pub fn from_u64_seed(
        seed: u64,
        height_wave_length: f64,
        height_amplitude: f64,
        height_octaves: usize,
    ) -> Self {
        let rng = ChaCha20Rng::seed_from_u64(seed);
        Self::new(rng, height_wave_length, height_amplitude, height_octaves)
    }
    pub fn from_entropy(
        height_wave_length: f64,
        height_amplitude: f64,
        height_octaves: usize,
    ) -> Self {
        let rng = ChaCha20Rng::from_entropy();
        Self::new(rng, height_wave_length, height_amplitude, height_octaves)
    }
    pub fn from_seed(
        seed: [u8; 32],
        height_wave_length: f64,
        height_amplitude: f64,
        height_octaves: usize,
    ) -> Self {
        let rng = ChaCha20Rng::from_seed(seed);
        Self::new(rng, height_wave_length, height_amplitude, height_octaves)
    }
    pub fn get_seed(&self)->[u8;32]{
        self.seed
    }
    pub fn new(
        rng: ChaCha20Rng,
        height_wave_length: f64,
        height_amplitude: f64,
        height_octaves: usize,
    ) -> Self {

        let mut height_rng = rng.clone();
        height_rng.set_stream(1);
        Self {
            seed:rng.get_seed(),
            height_generator: NoiseGenerator::from_rng(
                height_rng,
                1. / height_wave_length,
                height_amplitude,
                height_octaves,
            ),
        }
    }
    pub fn get_height(&mut self, x: usize) -> f64 {
        self.height_generator.sample(x)
    }
}
