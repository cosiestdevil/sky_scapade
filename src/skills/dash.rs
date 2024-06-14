use bevy::prelude::*;
use bevy_ecs::system::Query;
use bevy_tnua::{
    builtins::{TnuaBuiltinDash, TnuaBuiltinWalk},
    controller::TnuaController,
};
use leafwing_input_manager::action_state::ActionState;

use crate::{input::*, AppState, InGameState};
pub struct DashPlugin;
#[derive(Component)]
pub struct DashSkillDisplay;
impl Plugin for DashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (dash, dash_cooldown,dash_skill_display)
                .run_if(in_state(AppState::InGame).and_then(in_state(InGameState::Playing))),
        );
    }
}

fn dash(mut query: Query<(&ActionState<Action>, &mut TnuaController, &mut Player)>) {
    let (action_state, mut controller, mut player) = query.single_mut();

    if controller.dynamic_basis().is_some() && (!controller.is_airborne().unwrap() || player.dash_skill.air)
            && player.dash_skill.max_dash > player.used_dashes && action_state.just_pressed(&Action::Dash) {
        let basis: Option<(&TnuaBuiltinWalk, &_)> = controller.concrete_basis();
        if let Some(walk) = basis {
            player.used_dashes += 1;
            let direction = walk.0.desired_forward;
            if player.dash_cooldown.is_none() {
                player.dash_cooldown = Some(Timer::new(player.dash_skill.cooldown, TimerMode::Once))
            }

            controller.action(TnuaBuiltinDash {
                displacement: direction.normalize_or_zero() * player.max_speed() * 0.75,
                speed: player.max_speed() * 3.,
                allow_in_air: player.dash_skill.air,
                brake_to_speed: player.max_speed(),
                ..default()
            });
        }
    }
}
fn dash_cooldown(mut player: Query<&mut Player>, time: Res<Time>) {
    let mut player = player.single_mut();
    if let Some(ref mut cooldown) = player.dash_cooldown {
        cooldown.tick(time.delta());
        if cooldown.just_finished() {
            player.used_dashes -= 1;
            if player.used_dashes == 0 {
                player.dash_cooldown = None;
            } else {
                player.dash_cooldown =
                    Some(Timer::new(player.dash_skill.cooldown, TimerMode::Once));
            }
        }
    }
}
fn dash_skill_display(
    player: Query<&Player>,
    mut dashses: Query<&mut Text, With<DashSkillDisplay>>,
) {
    let player = player.single();
    if let Ok(mut dashses_text) = dashses.get_single_mut() {
        let air = if player.dash_skill.air { " (Air)" } else { "" };
        dashses_text.sections[0].value = format!(
            "Dash: {}{}",
            player.dash_skill.max_dash - player.used_dashes,
            air
        );
    }
}