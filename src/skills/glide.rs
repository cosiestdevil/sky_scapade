use action::GlideAction;
use bevy::prelude::*;
use bevy_ecs::system::Query;
use bevy_tnua::controller::TnuaController;
use leafwing_input_manager::action_state::ActionState;

use crate::{input::*, AppState, InGameState};
pub struct GlidePlugin;
impl Plugin for GlidePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (glide_start,glide_cooldown,glide_skill_display)
                .run_if(in_state(AppState::InGame).and_then(in_state(InGameState::Playing))),
        );
    }
}
#[derive(Component)]
pub struct Glider;
fn glide_start(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &ActionState<Action>,
        &mut TnuaController,
        &mut Player,
    )>,
    asset_server: ResMut<AssetServer>,
    glider_query: Query<Entity, With<Glider>>,
    time: Res<Time>,
) {
    let (player_entity, action_state, mut controller, mut player) = query.single_mut();
    if controller.dynamic_basis().is_none() {
        return;
    }
    if controller.is_airborne().unwrap()
        && action_state.just_pressed(&Action::Glide)
        && player.glide_timer.is_none()
        && player.used_glides < player.glide_skill.max_uses
    {
        player.used_glides += 1;
        player.glide_timer = Some(Timer::new(player.glide_skill.max_duration, TimerMode::Once));
        let mut player_entity = commands.entity(player_entity);
        let glider_scene: Handle<Scene> = asset_server.load("glider.glb#Scene0");
        player_entity.with_children(|child| {
            child.spawn((
                Glider,
                SceneBundle {
                    scene: glider_scene,
                    transform: Transform::from_xyz(0., 2., 0.),
                    ..default()
                },
            ));
        });
        if player.glide_cooldown.is_none() {
            player.glide_cooldown = Some(Timer::new(player.glide_skill.cooldown, TimerMode::Once))
        }
    }
    if let Some(timer) = &mut player.glide_timer {
        if controller.is_airborne().unwrap()
            && action_state.pressed(&Action::Glide)
            && !timer.finished()
        {
            controller.action(GlideAction);
            timer.tick(time.delta());
        } else {
            player.glide_timer = None;
            if let Ok(glider) = glider_query.get_single() {
                commands.entity(glider).despawn_recursive();
            }
        }
    }
}
fn glide_cooldown(mut player: Query<&mut Player>, time: Res<Time>) {
    let mut player = player.single_mut();
    if let Some(ref mut cooldown) = player.glide_cooldown {
        cooldown.tick(time.delta());
        if cooldown.just_finished() {
            player.used_glides -= 1;
            if player.used_glides == 0 {
                player.glide_cooldown = None;
            } else {
                player.glide_cooldown =
                    Some(Timer::new(player.glide_skill.cooldown, TimerMode::Once));
            }
        }
    }
}
#[derive(Component)]
pub struct GlideSkillDisplay;
fn glide_skill_display(
    player: Query<&Player>,
    mut jumps: Query<&mut Text, With<GlideSkillDisplay>>,
) {
    let player = player.single();
    if let Ok(mut jumps_text) = jumps.get_single_mut() {
        jumps_text.sections[0].value = format!(
            "Glide: {}",
            player.glide_skill.max_uses - player.used_glides
        );
    }
}

mod action {
    use bevy::prelude::*;
    use bevy_tnua::{
        math::AdjustPrecision, TnuaAction, TnuaActionLifecycleDirective, TnuaActionLifecycleStatus,
    };
    use strum::EnumCount;
    #[derive(Debug, Component)]
    pub struct GlideAction;
    impl TnuaAction for GlideAction {
        const NAME: &'static str = "SkyScapadeGlide";

        type State = GlideActionState;

        const VIOLATES_COYOTE_TIME: bool = false;

        fn apply(
            &self,
            state: &mut Self::State,
            ctx: bevy_tnua::TnuaActionContext,
            lifecycle_status: bevy_tnua::TnuaActionLifecycleStatus,
            motor: &mut bevy_tnua::TnuaMotor,
        ) -> bevy_tnua::TnuaActionLifecycleDirective {
            let up = ctx.up_direction().adjust_precision();

            if lifecycle_status.just_started() {
                *state = Self::State::StartingGlide;
            }
            let effective_velocity = ctx.basis.effective_velocity();
            for _ in 0..Self::State::COUNT {
                return match state {
                    GlideActionState::NoJump => todo!(),
                    GlideActionState::StartingGlide => {
                        *state = if lifecycle_status.is_active() {
                            GlideActionState::MaintainingGlide
                        } else {
                            GlideActionState::StoppedMaintainingGlide
                        };
                        lifecycle_status.directive_simple()
                    }
                    GlideActionState::MaintainingGlide => {
                        
                        let landed = ctx
                            .basis
                            .displacement()
                            .map_or(false, |displacement| displacement.dot(up) <= 0.0);
                        if landed
                            || matches!(lifecycle_status, TnuaActionLifecycleStatus::CancelledInto)
                        {
                            TnuaActionLifecycleDirective::Finished
                        } else {
                            *state = if lifecycle_status.is_active() {
                                GlideActionState::MaintainingGlide
                            } else {
                                GlideActionState::StoppedMaintainingGlide
                            };
                            motor.lin.cancel_on_axis(up);
                            motor.lin.acceleration += 9. * up;
                            TnuaActionLifecycleDirective::StillActive
                        }
                    }
                    GlideActionState::StoppedMaintainingGlide => {
                        if matches!(lifecycle_status, TnuaActionLifecycleStatus::CancelledInto) {
                            TnuaActionLifecycleDirective::Finished
                        } else {
                            let landed = ctx
                                .basis
                                .displacement()
                                .map_or(false, |displacement| displacement.dot(up) <= 0.0);
                            if landed {
                                TnuaActionLifecycleDirective::Finished
                            } else {
                                let upward_velocity = up.dot(effective_velocity);
                                if upward_velocity <= 0.0 {
                                    *state = GlideActionState::FallSection;
                                    continue;
                                }
                                TnuaActionLifecycleDirective::StillActive
                            }
                        }
                    }
                    GlideActionState::FallSection => {
                        let landed = ctx
                            .basis
                            .displacement()
                            .map_or(false, |displacement| displacement.dot(up) <= 0.0);
                        if landed
                            || matches!(lifecycle_status, TnuaActionLifecycleStatus::CancelledInto)
                        {
                            TnuaActionLifecycleDirective::Finished
                        } else {
                            motor.lin.cancel_on_axis(up);
                            motor.lin.acceleration -= 20. * up;
                            TnuaActionLifecycleDirective::StillActive
                        }
                    }
                };
            }
            error!("Tnua could not decide on glide state");
            TnuaActionLifecycleDirective::Finished
        }

        fn initiation_decision(
            &self,
            ctx: bevy_tnua::TnuaActionContext,
            _being_fed_for: &bevy::time::Stopwatch,
        ) -> bevy_tnua::TnuaActionInitiationDirective {
            if ctx.basis.is_airborne() {
                bevy_tnua::TnuaActionInitiationDirective::Allow
            } else {
                bevy_tnua::TnuaActionInitiationDirective::Reject
            }
        }
    }
    #[derive(Default, Debug, EnumCount)]
    pub enum GlideActionState {
        #[default]
        NoJump,
        // FreeFall,
        StartingGlide,
        MaintainingGlide,
        StoppedMaintainingGlide,
        FallSection,
    }
}
