use core::panic;

use bevy::log::info;
use bevy::utils::info;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
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
    #[derive(EnumIter, EnumCount, Debug, PartialEq, Copy, Clone, PartialOrd)]
    enum UpgradeLevel {
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
    #[derive(Debug, PartialEq, Copy, Clone)]
    struct Upgrade {
        upgrade_type: UpgradeType,
        upgrade_level: UpgradeLevel,
    }
    impl crate::upgrades::Upgrade<Upgrade> for Upgrade {
        fn is_lower(&self, other: Upgrade) -> bool {
            self.upgrade_type == other.upgrade_type && self.upgrade_level <= other.upgrade_level
        }
    }
}
