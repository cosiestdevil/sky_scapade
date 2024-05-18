use std::time::Duration;

use bevy_ecs::system::Resource;
use cosiest_noisiest::NoiseGenerator;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use strum::IntoEnumIterator;

use crate::{upgrades, StatUpgrade, UpgradeType};

#[derive(Resource, Clone)]
pub struct Generator {
    seed: [u8; 32],
    height_generator: cosiest_noisiest::NoiseGenerator<f64>,
    hole_generator: cosiest_noisiest::NoiseGenerator<f64>,
    upgrades: upgrades::WeightedUpgrades<ChaCha20Rng, UpgradeType>,
}

pub struct NoiseSettings {
    wave_length: f64,
    amplitude: f64,
    octaves: usize,
}
impl NoiseSettings {
    pub fn new(wave_length: impl Into<f64>, amplitude: impl Into<f64>, octaves: usize) -> Self {
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
    pub fn from_entropy(
        height_noise_settings: NoiseSettings,
        hole_noise_settings: NoiseSettings,

    ) -> Self {
        let rng = ChaCha20Rng::from_entropy();
        Self::new(rng, height_noise_settings, hole_noise_settings)
    }
    #[allow(dead_code)]
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
            upgrades: upgrades::WeightedUpgrades::new(upgrade_rng),
        };

        let weight_offset = 0.2;
        let mut weight = 100.;
        for (i,upgrade_level) in crate::UpgradeLevel::iter().enumerate() {
            if upgrade_level == crate::UpgradeLevel::None{
                continue;
            }
            result.upgrades.add_upgrade(
                UpgradeType::Speed(StatUpgrade {
                    modifier: 1. + (0.1* (i as f32)),
                    additive: false,
                    tier: upgrade_level,
                }),
                weight,
            );
            result.upgrades.add_upgrade(
                UpgradeType::JumpPower(StatUpgrade {
                    modifier: 0.5* (i as f32),
                    additive: true,
                    tier: upgrade_level,
                }),
                weight,
            );
            weight *= weight_offset;
        }

        result.upgrades.add_upgrade(UpgradeType::JumpSkill(crate::JumpSkill{
            max_jumps:2,
            tier:crate::UpgradeLevel::Advanced,
            air:true
        }), 50.);
        result.upgrades.add_upgrade(UpgradeType::JumpSkill(crate::JumpSkill{
            max_jumps:3,
            tier:crate::UpgradeLevel::Enhanced,
            air:true
        }), 10.);

        result.upgrades.add_upgrade(
            UpgradeType::DashSkill(crate::DashSkill {
                max_dash: 1,
                tier: crate::UpgradeLevel::Basic,
                air:false,
                cooldown:Duration::from_secs(8)
            }),
            100.,
        );
        result.upgrades.add_upgrade(
            UpgradeType::DashSkill(crate::DashSkill {
                max_dash: 2,
                tier: crate::UpgradeLevel::Improved,
                air:false,
                cooldown:Duration::from_secs(8)
            }),
            80.,
        );
        result.upgrades.add_upgrade(
            UpgradeType::DashSkill(crate::DashSkill {
                max_dash: 2,
                tier: crate::UpgradeLevel::Enhanced,
                air:false,
                cooldown:Duration::from_secs(7)
            }),
            64.,
        );
        result.upgrades.add_upgrade(
            UpgradeType::DashSkill(crate::DashSkill {
                max_dash: 2,
                tier: crate::UpgradeLevel::Advanced,
                air:true,
                cooldown:Duration::from_secs(7)
            }),
            51.,
        );
        result
    }
    pub fn get_height(&mut self, x: usize) -> f64 {
        self.height_generator.sample(x)
    }

    pub fn get_upgrade(&mut self) -> Option<UpgradeType> {
        self.upgrades.get_upgrade()
    }

    pub fn is_hole(&mut self, x: usize) -> bool {
        self.hole_generator.sample(x) >= self.hole_generator.amplitude * 0.95
    }
}
