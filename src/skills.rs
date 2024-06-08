use bevy::prelude::*;
mod dash;

pub struct SkillPlugin;

impl Plugin for SkillPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(dash::DashPlugin);
    }
}
