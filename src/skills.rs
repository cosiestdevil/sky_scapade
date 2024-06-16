use bevy::prelude::*;
mod dash;
mod glide;
pub mod jump;
pub use glide::GlideSkillDisplay;
pub use dash::DashSkillDisplay;
pub struct SkillPlugin;

impl Plugin for SkillPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(dash::DashPlugin);
        app.add_plugins(glide::GlidePlugin);
    }
}