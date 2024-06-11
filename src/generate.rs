use std::time::Duration;

use bevy_ecs::system::Resource;
use cosiest_noisiest::{Frequency, NoiseGenerator};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use strum::{EnumCount, IntoEnumIterator};

use crate::upgrades::*;

#[derive(Resource, Clone)]
pub struct Generator {
    seed: [u8; 32],
    height_generator: cosiest_noisiest::NoiseGenerator<f64>,
    hole_generator: cosiest_noisiest::NoiseGenerator<f64>,
    upgrades: WeightedUpgrades<ChaCha20Rng, UpgradeType>,
}

pub struct NoiseSettings {
    wave_length: usize,
    amplitude: f64,
    octaves: usize,
}
impl NoiseSettings {
    pub fn new(wave_length: impl Into<usize>, amplitude: impl Into<f64>, octaves: usize) -> Self {
        Self {
            wave_length: wave_length.into(),
            amplitude: amplitude.into(),
            octaves,
        }
    }
}

impl Generator {
    #[allow(dead_code)]
    pub fn from_u64_seed(
        seed: u64,
        height_noise_settings: NoiseSettings,
        hole_noise_settings: NoiseSettings,
    ) -> Self {
        let rng = ChaCha20Rng::seed_from_u64(seed);
        Self::new(rng, height_noise_settings, hole_noise_settings)
    }
    #[allow(dead_code)]
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
        let mut result = Self {
            seed: rng.get_seed(),
            height_generator: NoiseGenerator::from_rng(
                height_rng,
                Frequency::from_wave_length(height_noise_settings.wave_length),
                height_noise_settings.amplitude,
                height_noise_settings.octaves,
            ),
            hole_generator: NoiseGenerator::from_rng(
                hole_rng,
                Frequency::from_wave_length(hole_noise_settings.wave_length),
                hole_noise_settings.amplitude,
                hole_noise_settings.octaves,
            ),
            upgrades: WeightedUpgrades::new(upgrade_rng),
        };

        let weight_offset = 0.2;
        let mut weight = 50_000_000.;
        for (i, upgrade_level) in crate::upgrades::UpgradeLevel::iter().enumerate() {
            if upgrade_level == crate::upgrades::UpgradeLevel::None {
                continue;
            }
            result.upgrades.add_upgrade(
                UpgradeType::Speed(StatUpgrade {
                    stat:"Movement Speed",
                    modifier: 1. + (0.1 * (i as f32)),
                    additive: false,
                    tier: upgrade_level,
                }),
                weight,
            );
            result.upgrades.add_upgrade(
                UpgradeType::JumpPower(StatUpgrade {
                    stat:"Jump Height",
                    modifier: 0.5 * (i as f32),
                    additive: true,
                    tier: upgrade_level,
                }),
                weight,
            );
            if upgrade_level != UpgradeLevel::Basic {
                if i % (UpgradeLevel::COUNT / 4) == 0 {
                    result.upgrades.add_upgrade(
                        UpgradeType::GlideSkill(crate::upgrades::GlideSkill {
                            max_uses: 1 + (i / 4) as u8,
                            tier: upgrade_level,
                            cooldown: Duration::from_secs(10),
                            max_duration: Duration::from_secs(2),
                        }),
                        weight,
                    );
                }
                if i % (UpgradeLevel::COUNT / 3) == 0 {
                    result.upgrades.add_upgrade(
                        UpgradeType::JumpSkill(crate::upgrades::JumpSkill {
                            max_jumps: 1 + (i / 3) as u8,
                            tier: upgrade_level,
                            air: true,
                        }),
                        weight,
                    );
                }
                if i % 2 == 0 {
                    result.upgrades.add_upgrade(
                        UpgradeType::DashSkill(crate::upgrades::DashSkill {
                            max_dash: 1 + (i / 4) as u8,
                            tier: upgrade_level,
                            air: upgrade_level > UpgradeLevel::Advanced,
                            cooldown: Duration::from_secs_f64(8. * ((UpgradeLevel::COUNT - i) as f64 / UpgradeLevel::COUNT as f64)),
                        }),
                        weight,
                    );
                }
            }
            weight *= weight_offset;
        }
        result
    }
    pub fn get_height(&mut self, x: usize) -> f64 {
        self.height_generator.sample(x)
    }
    pub fn get_heights(&mut self, start: usize) -> [f64; 1024] {
        let mut result = [0.0f64; 1024];
        self.height_generator.fill(start, &mut result);
        result
    }

    pub fn get_upgrade(&mut self) -> Option<UpgradeType> {
        self.upgrades.get_upgrade()
    }

    pub fn is_hole(&mut self, x: usize) -> bool {
        self.hole_generator.sample(x) >= self.hole_generator.amplitude * 0.95
    }
}
