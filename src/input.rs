use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{DashSkill, JumpSkill, StatUpgrade,GlideSkill,RevivalSkill};

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum Action {
    Left,
    Right,
    Move,
    Jump,
    Dash,
    Glide,
    Accept,
}
#[derive(Component,Default)]
pub struct Player{
    pub base_speed:f32,
    pub speed_modifiers:Vec<StatUpgrade>,
    pub base_jump_power:f32,
    pub jump_modifiers:Vec<StatUpgrade>,
    pub score:f32,
    pub jump_skill:JumpSkill,
    pub dash_skill:DashSkill,
    pub used_dashes:u8,
    pub dash_cooldown:Option<Timer>,
    pub glide_skill:GlideSkill,
    pub used_glides:u8,
    pub glide_cooldown:Option<Timer>,
    pub glide_timer:Option<Timer>,
    pub revival_skill:RevivalSkill,
    pub used_revivals:u8
    }
impl Player{
    pub fn max_speed(&mut self)->f32{
        let mut result = self.base_speed;
        self.speed_modifiers.sort_unstable_by(|a,b| a.additive.cmp(&b.additive));
        for modifier in &self.speed_modifiers  {
            if modifier.additive{
                result += modifier.modifier;
            }else{
                result *= modifier.modifier;
            }
        }
        result
    }
    pub fn jump_power(&mut self)->f32{
        let mut result = self.base_jump_power;
        self.jump_modifiers.sort_unstable_by(|a,b| a.additive.cmp(&b.additive));
        for modifier in &self.jump_modifiers  {
            if modifier.additive{
                result += modifier.modifier;
            }else{
                result *= modifier.modifier;
            }
        }
        result
    }
}