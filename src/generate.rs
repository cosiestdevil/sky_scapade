use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use splines::{Interpolation, Key, Spline};

pub struct Generator {
    height_rng: ChaCha20Rng,
    height_wave_length: u64,
    height_amplitude: u64,
    height_octaves: u64,
}

impl Generator {
    pub fn new(
        seed: u64,
        height_wave_length: u64,
        height_amplitude: u64,
        height_octaves: u64,
    ) -> Self {
        let rng = ChaCha20Rng::seed_from_u64(seed);
        let mut height_rng = rng.clone();
        height_rng.set_stream(1);
        Self {
            height_rng,
            height_wave_length,
            height_amplitude,
            height_octaves,
        }
    }
    fn gen_noise(rng: &mut ChaCha20Rng, x: u64, wl: u64, amp: u64) -> f64 {
        let i = x / wl;
        let y: f64;
        if i < 1 {
            rng.set_word_pos((i * 2).into());
            y = rng.gen();
        } else {
            let previous_word_offset = (i - 1) * 2;
            println!("X = {}, prev = {}", x, previous_word_offset);
            rng.set_word_pos(previous_word_offset.into());
            let a: f64 = rng.gen();
            let b: f64 = rng.gen();
            y = Self::interpolate(a, b, (x % wl) as f64 / wl as f64);
        }
        y * (amp as f64)
    }
    pub fn get_height(&mut self, x: u64) -> f64 {
        let mut rng = self.height_rng.clone();
        let mut wl = self.height_wave_length;
        let mut amp = self.height_amplitude;
        let mut y = 0.;
        for _ in 0..self.height_octaves {
            if wl > 0 && amp > 0 {
                y += Self::gen_noise(&mut rng, x, wl, amp);
                wl /= 2;
                amp /= 2;
            }
        }
        y
    }
    fn interpolate(pa: f64, pb: f64, px: f64) -> f64 {
        let spline = Spline::from_vec(vec![
            Key::new(0., pa, Interpolation::Cosine),
            Key::new(1., pb, Interpolation::default()),
        ]);
        match spline.sample(px) {
            Some(sample) => sample,
            None => pa,
        }
        
    }
}
