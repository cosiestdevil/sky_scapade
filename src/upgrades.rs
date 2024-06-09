use core::panic;
use std::time::Duration;

use bevy::{log::info, render::color::Color};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use strum::{EnumCount, EnumIter};
use std::fmt::Debug;

#[derive(Clone)]
pub struct WeightedUpgrades<R: Rng + Sized, T: Upgrade<T>> {
    rng: R,
    upgrades: Vec<(Option<T>, f64)>,
    dist: WeightedIndex<f64>,
}

impl<R: Rng + Sized, T: Upgrade<T>+Debug> WeightedUpgrades<R, T> {
    pub fn new(rng: R) -> Self {
        Self {
            rng,
            upgrades: vec![(None, f32::EPSILON as f64)],
            dist: WeightedIndex::new([f32::EPSILON as f64]).unwrap(),
        }
    }
    pub fn add_upgrade(&mut self, upgrade: T, weight: f64) {
        self.upgrades.push((Some(upgrade), weight));
        self.dist = WeightedIndex::new(self.upgrades.iter().map(|item| item.1)).unwrap();
    }
    pub fn get_upgrade(&mut self) -> Option<T> {
        let value = self.dist.sample(&mut self.rng);
        let upgrade = self.upgrades[value].0;
        if let Some(upgrade) = upgrade {
            let mut removed = Vec::new();
            for (i, el) in self.upgrades.iter().enumerate() {
                if let Some(up) = el.0 {
                    if up.is_lower(upgrade) {
                        removed.push((i, &0.0))
                    }
                }
            }
            if let Err(err) = self.dist.update_weights(removed.as_slice()){
                info!("Error while removing {:?} from upgrade pool",removed);
                info!("{:?}",upgrade);
                panic!("{}",err);
            }
        }
        upgrade
    }
}

pub trait Upgrade<T: Upgrade<T>>: Copy + Clone {
    fn is_lower(&self, other: T) -> bool;
    fn tier(&self)->UpgradeLevel;
    fn color(&self) -> Color;
}
#[derive(Debug, Copy, Clone)]
pub enum UpgradeType {
    Speed(StatUpgrade),
    JumpPower(StatUpgrade),
    JumpSkill(JumpSkill),
    DashSkill(DashSkill),
    GlideSkill(GlideSkill),
}


#[derive(EnumIter, EnumCount, Debug, PartialEq, Copy, Clone, PartialOrd, Default)]
pub enum UpgradeLevel {
    #[default]
    None,
    Basic,
    Improved,
    Enhanced,
    Advanced,
    Superior,
    Elite,
    Master,
    Epic,
    Legendary,
    Mythic,
}
impl UpgradeLevel{
    pub fn color(&self) -> Color {
        match self{
            UpgradeLevel::None => Color::WHITE,
            UpgradeLevel::Basic => Color::GRAY,
            UpgradeLevel::Improved => Color::ALICE_BLUE,
            UpgradeLevel::Enhanced => Color::BLUE,
            UpgradeLevel::Advanced => Color::LIME_GREEN,
            UpgradeLevel::Superior => Color::GREEN,
            UpgradeLevel::Elite => Color::PINK,
            UpgradeLevel::Master => Color::PURPLE,
            UpgradeLevel::Epic => Color::GOLD,
            UpgradeLevel::Legendary => Color::ORANGE_RED,
            UpgradeLevel::Mythic => Color::RED,
        }
    }
    pub fn name(&self) -> String{
        match self {
            UpgradeLevel::None =>"None",
            UpgradeLevel::Basic => "Basic",
            UpgradeLevel::Improved => "Improved",
            UpgradeLevel::Enhanced => "Enhanced",
            UpgradeLevel::Advanced => "Advanced",
            UpgradeLevel::Superior => "Suprerior",
            UpgradeLevel::Elite => "Elite",
            UpgradeLevel::Master => "Master",
            UpgradeLevel::Epic => "Epic",
            UpgradeLevel::Legendary => "Legendary",
            UpgradeLevel::Mythic => "Mythic",
        }.into()
    }
}
#[derive(Debug, Copy, Clone, Default)]
pub struct JumpSkill {
    pub max_jumps: u8,
    pub tier: UpgradeLevel,
    pub air: bool,
}
#[derive(Debug, Copy, Clone, Default)]
pub struct DashSkill {
    pub max_dash: u8,
    pub air: bool,
    pub cooldown: Duration,
    pub tier: UpgradeLevel,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct GlideSkill {
    pub max_uses: u8,
    pub cooldown: Duration,
    pub tier: UpgradeLevel,
    pub max_duration: Duration,
}

#[derive(Debug, Copy, Clone)]
pub struct StatUpgrade {
    pub modifier: f32,
    pub additive: bool,
    pub tier: UpgradeLevel,
}

impl Upgrade<UpgradeType> for UpgradeType {
    fn is_lower(&self, other: UpgradeType) -> bool {
        match self {
            UpgradeType::Speed(me) => match other {
                UpgradeType::Speed(other) => me.tier <= other.tier,
                _ => false,
            },
            UpgradeType::JumpPower(me) => match other {
                UpgradeType::JumpPower(other) => me.tier <= other.tier,
                _ => false,
            },
            UpgradeType::JumpSkill(me) => match other {
                UpgradeType::JumpSkill(other) => me.tier <= other.tier,
                _ => false,
            },
            UpgradeType::DashSkill(me) => match other {
                UpgradeType::DashSkill(other) => me.tier <= other.tier,
                _ => false,
            },
            UpgradeType::GlideSkill(me) => match other {
                UpgradeType::GlideSkill(other) => me.tier <= other.tier,
                _ => false,
            },
        }
    }
    
    fn color(&self) -> Color {
        self.tier().color()
    }
    
    fn tier(&self)->UpgradeLevel {
        match self{
            UpgradeType::Speed(speed) => speed.tier,
            UpgradeType::JumpPower(jump) => jump.tier,
            UpgradeType::JumpSkill(jump) => jump.tier,
            UpgradeType::DashSkill(dash) => dash.tier,
            UpgradeType::GlideSkill(glide) => glide.tier,
        }
    }
    
    
}



#[cfg(test)]
mod tests {
    use super::*;
    use rand_chacha::ChaCha20Rng;
    use strum::*;
    fn setup(seed: u64, weight_ofsset: f64) -> WeightedUpgrades<ChaCha20Rng, Upgrade> {
        let mut weighted_upgrades = WeightedUpgrades::new(ChaCha20Rng::seed_from_u64(seed));
        let base_weight = 100.;
        for upgrade_type in UpgradeType::iter() {
            let mut weight = base_weight;
            for upgrade_level in UpgradeLevel::iter() {
                if upgrade_level== UpgradeLevel::None{
                    continue;
                }
                weighted_upgrades.add_upgrade(
                    Upgrade {
                        upgrade_level,
                        upgrade_type,
                    },
                    weight,
                );
                weight *= weight_ofsset;
            }
        }
        weighted_upgrades
    }
    #[test]
    fn it_works() {
        let mut weighted_upgrades = setup(0, 0.2);
        let result = weighted_upgrades.get_upgrade();
        assert_eq!(
            result,
            Some(Upgrade {
                upgrade_type: UpgradeType::Speed,
                upgrade_level: UpgradeLevel::Basic
            })
        )
    }
    #[test]
    fn other_test() {
        let mut weighted_upgrades = setup(0, 5.0);
        let result = weighted_upgrades.get_upgrade();
        assert_eq!(
            result,
            Some(Upgrade {
                upgrade_type: UpgradeType::Speed,
                upgrade_level: UpgradeLevel::Legendary
            })
        )
    }
    #[test]
    fn different_seed() {
        let mut weighted_upgrades = setup(1, 0.2);
        let result = weighted_upgrades.get_upgrade();
        assert_ne!(
            result,
            Some(Upgrade {
                upgrade_type: UpgradeType::Speed,
                upgrade_level: UpgradeLevel::Basic
            })
        )
    }
    #[derive(EnumIter, EnumCount, Debug, PartialEq, Copy, Clone)]
    enum UpgradeType {
        Speed,
        Jump,
        Double,
    }
    #[derive(Debug, PartialEq, Copy, Clone)]
    struct Upgrade {
        upgrade_type: UpgradeType,
        upgrade_level: UpgradeLevel,
    }
    impl crate::upgrades::Upgrade<Upgrade> for Upgrade {
        fn is_lower(&self, other: Upgrade) -> bool {
            self.upgrade_type == other.upgrade_type && self.upgrade_level <= other.upgrade_level
        }
        
        fn tier(&self)->crate::UpgradeLevel {
            self.upgrade_level
        }
        
        fn color(&self) -> Color {
            Color::WHITE
        }
    }
}
