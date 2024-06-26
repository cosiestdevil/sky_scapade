#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#[cfg(not(feature = "bevy_mod_taa"))]
use bevy::core_pipeline::experimental::taa::TemporalAntiAliasBundle as TAABundle;
use bevy::{
    asset::LoadState,
    audio::Volume,
    core_pipeline::Skybox,
    log,
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{
            Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
        }
    },
    window::PresentMode,
    winit::{UpdateMode, WinitSettings},
};
use bevy_ecs::system::EntityCommands;
use bevy_embedded_assets::EmbeddedAssetPlugin;
#[cfg(feature = "bevy_mod_taa")]
use bevy_mod_taa::TAABundle;
use bevy_obj::ObjPlugin;
use bevy_rapier3d::prelude::*;
use bevy_tnua::{
    builtins::{TnuaBuiltinJump, TnuaBuiltinWalk},
    control_helpers::TnuaSimpleAirActionsCounter,
    controller::{TnuaController, TnuaControllerBundle, TnuaControllerPlugin},
    TnuaAction,
};
use bevy_tnua_rapier3d::{TnuaRapier3dIOBundle, TnuaRapier3dPlugin, TnuaRapier3dSensorShape};

use leafwing_input_manager::{
    action_state::ActionState, axislike::DualAxis, input_map::InputMap, plugin::InputManagerPlugin,
    InputManagerBundle,
};
use std::{
    str::from_utf8,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
mod discord;
mod generate;
mod input;
mod menu;
mod settings;
mod skills;
mod system_info;
mod upgrades;
use crate::input::Player;
use crate::upgrades::*;
const GAME_NAME: &str = "SkyScapade";
fn main() {
    let mut app = App::new();
    app.add_plugins(EmbeddedAssetPlugin {
        mode: bevy_embedded_assets::PluginMode::ReplaceDefault,
    });
    app.add_plugins(discord::DiscordPlugin);
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: GAME_NAME.into(),
                    //resolution: (2560.0, 1080.0).into(),
                    resolution: (1280., 720.).into(),
                    name: Some("new_game_1.app".into()),
                    present_mode: PresentMode::AutoVsync,
                    visible: false,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    )
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
    //.add_plugins(RapierDebugRenderPlugin::default())
    .insert_resource(WinitSettings {
        focused_mode: UpdateMode::Continuous,
        unfocused_mode: UpdateMode::ReactiveLowPower {
            wait: Duration::from_secs_f64(1.0 / 30.0), //Duration::MAX
        },
    })
    .add_plugins(menu::MenuPlugin)
    .add_plugins(settings::SettingsPlugin)
    .add_plugins(ObjPlugin)
    .insert_state(AppState::MainMenu)
    //.add_plugins(ScreenDiagnosticsPlugin::default())
    //.add_plugins(ScreenFrameDiagnosticsPlugin)
    //.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
    //.add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
    //.add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
    //.add_plugins(system_info::SystemInformationDiagnosticsPlugin)
    .add_plugins(bevy_framepace::FramepacePlugin)
    //.add_plugins(PerfUiPlugin)
    .add_systems(Startup, setup)
    .add_systems(Update, (temp, skybox_loaded));
    app.insert_state(InGameState::Playing);
    app.add_plugins(InputManagerPlugin::<input::Action>::default());
    app.add_plugins((
        TnuaControllerPlugin::default(),
        TnuaRapier3dPlugin::default(),
    ));
    app.add_systems(OnExit(AppState::InGame), cleanup_level);
    app.add_systems(OnEnter(AppState::InGame), start_level);
    app.add_systems(
        Update,
        (
            move_player,
            move_camera_based_on_speed,
            upgrade_notification_timers,
        )
            .run_if(in_state(AppState::InGame).and_then(in_state(InGameState::Playing))),
    );
    app.add_plugins(skills::SkillPlugin);
    app.add_systems(
        FixedUpdate,
        (
            generate_more_if_needed,
            level_upgrade,
            level_finish,
            killing_floor,
            update_score,
            update_player_position_display,
            glide_cooldown,
            jump_skill_display,
            dash_skill_display,
            slow_fall_skill_display,
        )
            .run_if(in_state(AppState::InGame).and_then(in_state(InGameState::Playing))),
    );
    app.add_systems(
        Update,
        (accept_upgrade).run_if(in_state(InGameState::Upgrade)),
    );
    app.add_systems(OnEnter(InGameState::Paused), pause_level);
    app.add_systems(OnExit(InGameState::Paused), resume_level);
    app.add_systems(OnEnter(InGameState::Upgrade), pause_level);
    app.add_systems(OnExit(InGameState::Upgrade), resume_level);
    app.add_systems(OnEnter(InGameState::End), pause_level);
    app.add_systems(OnExit(InGameState::End), (resume_level, leave_end_screen));
    app.run();
}

#[derive(Resource, Clone)]
struct PlatformAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

fn pause_level(mut physics: ResMut<RapierConfiguration>) {
    physics.physics_pipeline_active = false;
}
fn resume_level(mut physics: ResMut<RapierConfiguration>) {
    physics.physics_pipeline_active = true;
}

fn move_camera_based_on_speed(
    mut query_camera: Query<(&mut Projection, &mut Transform), With<Camera>>,
    velocities: Query<&Velocity, With<Player>>,
) {
    let (projection, mut transform) = query_camera.single_mut();
    let Projection::Perspective(persp) = projection.into_inner() else {
        return;
    };
    let player_velocity = velocities.single();
    let min_fov = 45.;
    let max_fov = 120.;
    let fov_modifier = (player_velocity.linvel.x.abs().powf(0.125) / 8.).clamp(0., 1.);

    persp.fov = interpolate(min_fov, max_fov, fov_modifier).to_radians();
    let positive = player_velocity.linvel.x > 0.;
    transform.translation.x =
        (player_velocity.linvel.x.abs().sqrt() / 4.) * if positive { 1. } else { -1. };
}
fn interpolate(pa: f32, pb: f32, px: f32) -> f32 {
    let ft = px * std::f32::consts::PI;
    let f = (1. - ft.cos()) * 0.5;
    pa * (1. - f) + pb * f
}

fn level_finish(
    mut level: Query<&mut Level>,
    time: Res<Time>,
    player: Query<&Transform, With<Player>>,
    mut next_state: ResMut<NextState<InGameState>>,
    mut time_text: Query<&mut Text, With<TimeDisplay>>,
) {
    let mut level = level.single_mut();
    let player = player.single();
    level.timer.tick(time.delta());

    if let Ok(mut time_text) = time_text.get_single_mut() {
        let time = level.timer.elapsed().as_secs();
        time_text.sections[0].value = format!("Time: {:02}:{:02}", time / 60, time % 60);
    }
    if level.timer.just_finished() {
        log::info!("Level Finished. Travelled: {}", player.translation.x);
        next_state.set(InGameState::End);
    }
}

fn accept_upgrade(
    mut commands: Commands,
    mut next_state: ResMut<NextState<InGameState>>,
    action: Query<&ActionState<input::Action>>,
    screen: Query<Entity, With<UpgradeScreen>>,
) {
    let action_state = action.single();
    if action_state.just_pressed(&input::Action::Accept) {
        commands.entity(screen.single()).despawn_recursive();
        next_state.set(InGameState::Playing);
    }
}

fn leave_end_screen(mut commands: Commands, screen: Query<Entity, With<EndScreen>>) {
    commands.entity(screen.single()).despawn_recursive();
}

#[derive(Component)]
struct EndScreen;
#[derive(Component)]
struct UpgradeScreen {
    opening_timer: Timer,
    visible_timer: Timer,
    closing_timer: Timer,
}
impl UpgradeScreen {
    pub fn new() -> Self {
        Self {
            opening_timer: Timer::from_seconds(1., TimerMode::Once),
            visible_timer: Timer::from_seconds(5., TimerMode::Once),
            closing_timer: Timer::from_seconds(1., TimerMode::Once),
        }
    }
}
fn upgrade_notification_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut notifications: Query<(Entity, &mut UpgradeScreen, &mut Style)>,
) {
    for (notification_entity, mut notification, mut transform) in notifications.iter_mut() {
        if !notification.opening_timer.finished() {
            notification.opening_timer.tick(time.delta());
            transform.bottom = Val::Px(interpolate(
                -100.,
                0.,
                notification.opening_timer.fraction(),
            ));
        } else if !notification.visible_timer.finished() {
            notification.visible_timer.tick(time.delta());
        } else if !notification.closing_timer.finished() {
            notification.closing_timer.tick(time.delta());
            transform.bottom = Val::Px(interpolate(
                0.,
                -100.,
                notification.closing_timer.fraction(),
            ));
        } else {
            let notification_entity = commands.entity(notification_entity);
            notification_entity.despawn_recursive();
        }
    }
}

fn level_upgrade(
    mut commands: Commands,
    time: Res<Time>,
    safe_ui: Query<Entity, With<crate::SafeUi>>,
    mut level: Query<&mut Level>,
    mut player: Query<&mut Player>,
    mut generator: ResMut<generate::Generator>,
    asset_server: Res<AssetServer>,
) {
    let mut level = level.single_mut();
    level.upgrade_timer.tick(time.delta());
    if level.upgrade_timer.just_finished() {
        let upgrade = generator.get_upgrade();
        log::info!("Upgrade:{:?}", upgrade);
        if let Some(upgrade) = upgrade {
            let mut player = player.single_mut();
            let display;
            match upgrade {
                UpgradeType::Speed(upgrade) => {
                    player.speed_modifiers.push(upgrade);
                    display = format!("{} ({:?})", "Speed Upgrade", upgrade.tier);
                }
                UpgradeType::JumpPower(upgrade) => {
                    player.jump_modifiers.push(upgrade);
                    display = format!("{} ({:?})", "Jump Power Upgrade", upgrade.tier);
                }
                UpgradeType::JumpSkill(skill) => {
                    player.jump_skill = skill;
                    display = format!("{} ({:?})", "Extra Jump Upgrade", skill.tier);
                }
                UpgradeType::DashSkill(skill) => {
                    player.dash_skill = skill;
                    display = format!("{} ({:?})", "Dash Upgrade", skill.tier);
                }
                UpgradeType::GlideSkill(skill) => {
                    player.glide_skill = skill;
                    display = format!("{} ({:?})", "Glide Upgrade", skill.tier);
                }
            }
            commands.spawn(AudioBundle {
                source: asset_server.load("upgrade.mp3"),
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Once,
                    volume: Volume::new(0.4),
                    ..default()
                },
            });
            
            let safe_ui = safe_ui.get_single();
            if let Ok(safe_ui) = safe_ui {
                let mut safe_ui = commands.entity(safe_ui);
                safe_ui.with_children(|ui| {
                    ui.spawn((
                        NodeBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                width: Val::Percent(30.),
                                height: Val::Auto,
                                left: Val::Percent(37.),
                                ..default()
                            },
                            ..default()
                        },
                        UpgradeScreen::new(),
                    ))
                    .with_children(|screen| {
                        screen
                            .spawn(NodeBundle {
                                style: Style {
                                    display: Display::Grid,
                                    align_items: AlignItems::Center,
                                    justify_items: JustifyItems::Center,
                                    max_width: Val::Percent(100.),
                                    height: Val::Percent(100.),
                                    border: UiRect::all(Val::Px(5.)),
                                    padding:UiRect::all(Val::Px(5.)),
                                    ..default()
                                },
                                border_color:upgrade.color().into(),
                                background_color: Color::rgba(0., 0., 0., 0.6).into(),
                                ..default()
                            })
                            .with_children(|container| {
                                container.spawn(
                                    TextBundle::from_section(
                                        display,
                                        TextStyle {
                                            font_size: 20.,
                                            ..default()
                                        },
                                    )
                                    .with_text_justify(JustifyText::Center),
                                );
                            });
                    });
                });
                //next_state.set(InGameState::Upgrade);
            }
        }
    }
}

#[derive(Component)]
struct Score;
#[derive(Component)]
struct TimeDisplay;
fn update_score(
    //mut commands: Commands,
    mut player: Query<(&Transform, &mut Player)>,
    mut score: Query<&mut Text, With<Score>>,
) {
    let (player_transform, mut player) = player.single_mut();
    if player_transform.translation.x > player.score {
        player.score = player_transform.translation.x;
    }
    if let Ok(mut score_text) = score.get_single_mut() {
        score_text.sections[0].value = format!("Score: {:.0}", player.score);
    }
}

#[derive(Component)]
struct PositionDisplay;
fn update_player_position_display(
    mut player: Query<(&Transform, &Velocity), With<Player>>,
    mut score: Query<&mut Text, With<PositionDisplay>>,
) {
    let (player_transform, velocity) = player.single_mut();
    if let Ok(mut score_text) = score.get_single_mut() {
        score_text.sections[0].value = format!(
            "Position: [{:+5.1},{:+5.1},{:+5.1}]\r\nVelocity: [{:+5.1}mph,{:+5.1}mph,{:+5.1}mph]",
            player_transform.translation.x,
            player_transform.translation.y,
            player_transform.translation.z,
            velocity.linvel.x * 0.681818,
            velocity.linvel.y * 0.681818,
            velocity.linvel.z * 0.681818
        );
    }
}

fn generate_more_if_needed(
    mut commands: Commands,
    mut level: Query<(Entity, &mut crate::Level)>,
    platform_assets: Res<PlatformAssets>,
    player: Query<&Transform, With<Player>>,
    mut generator: ResMut<generate::Generator>,
) {
    let (level_entity, mut level) = level.single_mut();
    let cube_size = 1.0f32;
    let player_transform = player.single();
    if (player_transform.translation.x / cube_size) >= (level.right * 2) as f32 - 100. {
        let mut hole_streak = 0;
        let generate_offset = level.right;
        let heights = generator.get_heights(generate_offset);
        for (x, y) in heights.into_iter().enumerate() {
            let x = x + generate_offset;
            let platform_assets = platform_assets.clone();
            if hole_streak > 4 {
                hole_streak = 0;
            } else if generator.is_hole(x) {
                hole_streak += 1;
                continue;
            }
            let x: f32 = x as f32 * cube_size * 2.;
            let y = (y as f32) * cube_size;
            commands
                .spawn(Collider::cuboid(cube_size, cube_size, cube_size))
                .insert(PbrBundle {
                    mesh: platform_assets.mesh.clone(),
                    material: platform_assets.material.clone(),
                    ..default()
                })
                .insert(LevelFloor)
                .insert(TransformBundle::from_transform(Transform::from_xyz(
                    x, y, 0.,
                )))
                .set_parent(level_entity);
        }
        level.right += heights.len();
        info!("level.right: {}", level.right);
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
struct Glider;

fn move_player(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &ActionState<input::Action>,
        &mut TnuaController,
        &mut input::Player,
        &mut TnuaSimpleAirActionsCounter,
    )>,
    glider_query: Query<Entity, With<Glider>>,
    asset_server: ResMut<AssetServer>,
    time: Res<Time>,
) {
    let (player_entity, action_state, mut controller, mut player, mut air_actions_counter) =
        query.single_mut();
    // Each action has a button-like state of its own that you can check
    //println!("move_player {:?}",action_state);
    air_actions_counter.update(controller.as_mut());
    let mut direction = Vec3::ZERO;
    if action_state.pressed(&input::Action::Left) {
        direction -= Vec3::X;
    }
    if action_state.pressed(&input::Action::Right) {
        direction += Vec3::X;
    }
    if action_state.pressed(&input::Action::Move) {
        let xy = 0.2
            + action_state
                .clamped_axis_pair(&input::Action::Move)
                .unwrap()
                .xy();
        direction = Vec3::new(xy.x, 0., 0.)
    }
    controller.basis(TnuaBuiltinWalk {
        desired_velocity: direction.clamp(-Vec3::X, Vec3::X) * player.max_speed(),
        desired_forward: direction.normalize_or_zero(),
        float_height: 2.,
        ..Default::default()
    });
    if controller.is_airborne().unwrap()
        && action_state.pressed(&input::Action::Glide)
        && (match &player.glide_timer {
            Some(timer) => !timer.finished(),
            None => true,
        })
        && player.used_glides < player.glide_skill.max_uses
    {
        if action_state.just_pressed(&input::Action::Glide) {
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
        } else if let Some(timer) = &mut player.glide_timer {
            timer.tick(time.delta());
        }
        if player.glide_cooldown.is_none() {
            player.glide_cooldown = Some(Timer::new(player.glide_skill.cooldown, TimerMode::Once))
        }
        controller.action(TnuaBuiltinJump {
            height: 0.1,
            fall_extra_gravity: -5.,
            allow_in_air: true,
            ..default()
        });
    }
    let mut glide_over = player.glide_timer.is_some()
        && (!controller.is_airborne().unwrap()
            || action_state.just_released(&input::Action::Glide)
            || match &player.glide_timer {
                Some(timer) => timer.finished(),
                None => false,
            });
    if action_state.pressed(&input::Action::Jump) {
        let air_jumps: usize = (player.jump_skill.max_jumps - 1).into();
        glide_over = player.glide_timer.is_some();
        controller.action(TnuaBuiltinJump {
            height: player.jump_power(),
            allow_in_air: player.jump_skill.air
                && air_actions_counter.air_count_for(TnuaBuiltinJump::NAME) <= air_jumps,
            ..default()
        });
    }

    if glide_over {
        player.glide_timer = None;
        if let Ok(glider) = glider_query.get_single() {
            commands.entity(glider).despawn_recursive();
        }
        if !action_state.pressed(&input::Action::Jump) {
            controller.action(TnuaBuiltinJump {
                height: 0.1,
                fall_extra_gravity: 20.,
                allow_in_air: true,
                ..default()
            });
        }
    }
}

#[derive(Component)]
struct Level {
    right: usize,
    upgrade_timer: Timer,
    timer: Timer,
}

#[derive(Component)]
struct LevelFloor;
fn cleanup_level(mut commands: Commands, level: Query<Entity, With<Level>>) {
    let level = level.iter();
    for level in level {
        commands.entity(level).despawn_recursive();
    }
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::new(0., 0.0, 0.), Vec3::Y),
        ..default()
    });
}

fn killing_floor(
    mut commands: Commands,
    player: Query<(Entity, &Transform, &Player)>,
    mut next_state: ResMut<NextState<InGameState>>,
    mut generator: ResMut<generate::Generator>,
    //safe_ui: Query<Entity, With<crate::SafeUi>>,
) {
    let (_entity, player_transform, player) = player.single();
    let y = generator.get_height((player_transform.translation.x / 2.) as usize) as f32;
    if player_transform.translation.y < y - 10. {
        //let safe_ui = safe_ui.get_single();
        // if let Ok(safe_ui) = safe_ui {
        //     let mut safe_ui = commands.entity(safe_ui);
        //     safe_ui.with_children(|ui| {
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        display: Display::Grid,
                        align_items: AlignItems::Center,
                        justify_items: JustifyItems::Center,
                        ..default()
                    },
                    background_color: Color::rgba(0., 0., 0., 0.6).into(),
                    ..default()
                },
                EndScreen,
            ))
            .with_children(|screen| {
                screen.spawn(
                    TextBundle::from_section(
                        format!("Final Score\n{:.0}", player.score),
                        TextStyle {
                            font_size: 72.,
                            ..default()
                        },
                    )
                    .with_text_justify(JustifyText::Center),
                );
            });
        //});
        //}
        next_state.set(InGameState::End);
    }
}

#[derive(Component)]
struct DashSkillDisplay;
#[derive(Component)]
struct JumpSkillDisplay;
#[derive(Component)]
struct GlideSkillDisplay;

fn jump_skill_display(player: Query<&Player>, mut jumps: Query<&mut Text, With<JumpSkillDisplay>>) {
    let player = player.single();
    if let Ok(mut jumps_text) = jumps.get_single_mut() {
        jumps_text.sections[0].value = format!("Jump: {}", player.jump_skill.max_jumps);
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
fn slow_fall_skill_display(
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
type StartLevelAssets<'a> = (
    Res<'a, AssetServer>,
    ResMut<'a, Assets<Image>>,
    ResMut<'a, Assets<StandardMaterial>>,
    ResMut<'a, Assets<Mesh>>,
);
fn start_level(
    mut commands: Commands,
    mut camera: Query<(Entity, &mut Transform), With<Camera>>,
    safe_ui: Query<Entity, With<crate::SafeUi>>,
    assets: StartLevelAssets,
    mut next_state: ResMut<NextState<InGameState>>,
    mut discord_activity: ResMut<discord::ActivityState>,
    mut generator: ResMut<generate::Generator>,
) {
    next_state.set(InGameState::Playing);

    let (asset_server, mut images, mut materials, mut meshes) = assets;
    let platform_mesh: Handle<Mesh> = asset_server.load("platform.obj");
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });
    discord_activity.state = Some("Playing Solo".into());
    let seed = generator.get_seed();
    let seed: &str = from_utf8(&seed).unwrap();
    discord_activity.details = Some(format!("Seed: {}", seed));
    discord_activity.start = Some(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .try_into()
            .unwrap(),
    );
    let platform_assets = PlatformAssets {
        mesh: platform_mesh.clone(),
        material: debug_material.clone(),
    };
    commands.insert_resource(platform_assets);
    let safe_ui = safe_ui.get_single();
    if let Ok(safe_ui) = safe_ui {
        let mut safe_ui = commands.entity(safe_ui);
        safe_ui.with_children(|ui| {
            ui.spawn(NodeBundle {
                style: Style {
                    border: UiRect::all(Val::Px(1.)),
                    width: Val::Percent(100.),
                    display: Display::Grid,
                    grid_template_columns: vec![
                        GridTrack::auto(),
                        GridTrack::fr(1.0),
                        GridTrack::auto(),
                    ],
                    ..default()
                },
                //border_color:Color::RED.into(),
                ..default()
            })
            .with_children(|ui| {
                ui.spawn(NodeBundle {
                    style: Style {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        grid_column: GridPlacement::start(1),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|ui| {
                    ui.spawn(TextBundle::from_section(
                        format!("Seed: {}", seed),
                        TextStyle {
                            color: Color::WHITE,
                            font_size: 24.0,
                            ..default()
                        },
                    ));

                    ui.spawn(TextBundle::from_section(
                        format!("Score: {}", 0.0),
                        TextStyle {
                            color: Color::WHITE,
                            font_size: 24.0,
                            ..default()
                        },
                    ))
                    .insert(Score);
                    ui.spawn(TextBundle::from_section(
                        format!("Time: {:?}", Duration::from_secs(0)),
                        TextStyle {
                            color: Color::WHITE,
                            font_size: 24.0,
                            ..default()
                        },
                    ))
                    .insert(TimeDisplay);
                    ui.spawn(TextBundle::from_section(
                        "Position:".to_string(),
                        TextStyle {
                            color: Color::WHITE,
                            font_size: 24.0,
                            ..default()
                        },
                    ))
                    .insert(PositionDisplay);
                });
                ui.spawn(NodeBundle {
                    style: Style {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        grid_column: GridPlacement::start(3),
                        justify_content: JustifyContent::Center,
                        height: Val::Percent(100.),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|skills| {
                    skills.spawn((
                        TextBundle::from_section(
                            format!("Dash: {}", 0),
                            TextStyle {
                                color: Color::WHITE,
                                font_size: 24.0,
                                ..default()
                            },
                        ),
                        DashSkillDisplay,
                    ));
                    skills.spawn((
                        TextBundle::from_section(
                            format!("Jump: {}", 1),
                            TextStyle {
                                color: Color::WHITE,
                                font_size: 24.0,
                                ..default()
                            },
                        ),
                        JumpSkillDisplay,
                    ));
                    skills.spawn((
                        TextBundle::from_section(
                            format!("Glide: {}", 0),
                            TextStyle {
                                color: Color::WHITE,
                                font_size: 24.0,
                                ..default()
                            },
                        ),
                        GlideSkillDisplay,
                    ));
                });
            });
        });
    }
    let heights = generator.get_heights(0);
    let level = commands
        .spawn((
            Level {
                right: heights.len(),
                upgrade_timer: Timer::new(Duration::from_secs(10), TimerMode::Repeating),
                timer: Timer::new(Duration::from_secs(300), TimerMode::Once),
            },
            TransformBundle::default(),
            VisibilityBundle::default(),
        ))
        .id();
    info!("level.right: {}", heights.len());
    let cube_size = 1.0f32;

    commands
        .spawn(Collider::cuboid(cube_size, cube_size, cube_size))
        .insert(PbrBundle {
            mesh: platform_mesh.clone(),
            material: debug_material.clone(),
            ..default()
        })
        .insert(LevelFloor)
        .insert(TransformBundle::from_transform(Transform::from_xyz(
            0.,
            (heights[0] as f32) * cube_size,
            0.,
        )))
        .set_parent(level);
    let mut input_map = InputMap::default();
    input_map.insert(input::Action::Jump, KeyCode::Space);
    input_map.insert(input::Action::Jump, GamepadButtonType::South);
    input_map.insert(input::Action::Left, KeyCode::KeyA);
    input_map.insert(input::Action::Right, KeyCode::KeyD);
    input_map.insert(input::Action::Move, DualAxis::left_stick());
    input_map.insert(input::Action::Dash, KeyCode::ShiftLeft);
    input_map.insert(input::Action::Dash, GamepadButtonType::West);
    input_map.insert(input::Action::Accept, KeyCode::Enter);
    input_map.insert(input::Action::Accept, GamepadButtonType::South);
    input_map.insert(input::Action::Glide, KeyCode::KeyW);
    input_map.insert(input::Action::Glide, GamepadButtonType::North);

    let player_mesh = meshes.add(Capsule3d::new(0.4, 2.));
    let player = commands
        .spawn(Collider::capsule_y(1., 0.4))
        .insert(PbrBundle {
            mesh: player_mesh,
            material: debug_material.clone(),
            ..default()
        })
        .insert(TnuaRapier3dSensorShape(Collider::ball(0.4)))
        .insert(TnuaControllerBundle::default())
        .insert(TnuaRapier3dIOBundle::default())
        .insert(ColliderMassProperties::Density(1.0))
        .insert(input::Player {
            base_speed: 10.,
            base_jump_power: 5.,
            speed_modifiers: vec![],
            jump_modifiers: vec![],
            jump_skill: JumpSkill {
                max_jumps: 1,
                tier: UpgradeLevel::None,
                air: false,
            },
            ..default()
        })
        .insert(InputManagerBundle::with_map(input_map))
        .insert(TnuaSimpleAirActionsCounter::default())
        .insert(RigidBody::Dynamic)
        .insert(LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z)
        .insert(TransformBundle::from(Transform::from_xyz(
            1.5 * cube_size,
            (heights[0] as f32) + (3.5 * cube_size),
            0.,
        )))
        .set_parent(level)
        .id();
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
            color: Color::rgb_u8(234, 212, 165),
            ..default()
        },
        transform: Transform::from_xyz(10.0, 60.0, -10.0).looking_at(Vec3::splat(0.0), Vec3::Y),
        ..default()
    });

    if let Ok((camera, mut camera_transform)) = camera.get_single_mut() {
        // =
        commands.entity(camera).set_parent(player);
        *camera_transform =
            Transform::from_xyz(0.0, 5., 20.).looking_at(Vec3::new(0., 0., 0.), Vec3::Y)
    }
    for (x, hy) in heights.into_iter().enumerate().skip(1).take(5) {
        let hy = (hy as f32) * cube_size;
        commands
            .spawn(Collider::cuboid(cube_size, cube_size, cube_size))
            .insert(PbrBundle {
                mesh: platform_mesh.clone(),
                material: debug_material.clone(),
                ..default()
            })
            .insert(LevelFloor)
            .insert(TransformBundle::from_transform(Transform::from_xyz(
                (x as f32) * cube_size * 2.,
                hy,
                0.,
            )))
            .set_parent(level);
    }
    let mut hole_streak = 0;
    for (x, hy) in heights.into_iter().enumerate().skip(6) {
        if hole_streak > 4 {
            hole_streak = 0;
        } else if generator.is_hole(x) {
            hole_streak += 1;
            continue;
        }
        let x = (x as f32) * cube_size * 2.;
        let y = (hy as f32) * cube_size;
        commands
            .spawn(Collider::cuboid(cube_size, cube_size, cube_size))
            .insert(PbrBundle {
                mesh: platform_mesh.clone(),
                material: debug_material.clone(),
                ..default()
            })
            .insert(LevelFloor)
            .insert(TransformBundle::from_transform(Transform::from_xyz(
                x, y, 0.,
            )))
            .set_parent(level);
    }
}
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}
#[derive(Component)]
struct SafeUi;
trait UiHelper {
    fn new_menu_button(&mut self, label: &str, component: impl Bundle) -> EntityCommands;
}
impl UiHelper for ChildBuilder<'_> {
    fn new_menu_button(&mut self, label: &str, component: impl Bundle) -> EntityCommands {
        let mut result = self.spawn((
            ButtonBundle {
                style: Style {
                    display: Display::Grid,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::WHITE),
                border_color: Color::BLACK.into(),
                ..default()
            },
            component,
        ));
        result.with_children(|button| {
            button.spawn(
                TextBundle::from_section(
                    label,
                    TextStyle {
                        color: Color::RED,
                        font_size: 22.0,
                        ..default()
                    },
                )
                .with_text_justify(JustifyText::Center)
                .with_style(Style {
                    align_self: AlignSelf::Center,
                    justify_self: JustifySelf::Center,
                    ..default()
                }),
            );
        });
        result
    }
}

#[derive(Resource, Deref)]
pub struct SkyboxHandle(Handle<Image>);

fn skybox_loaded(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    skybox_handle: Res<SkyboxHandle>,
    camera: Query<Entity, (With<Camera>, Without<Skybox>)>,
) {
    for camera_entity in camera.iter() {
        let mut camera = commands.entity(camera_entity);
        if asset_server.load_state(&skybox_handle.0) == LoadState::Loaded {
            let image = images.get_mut(&skybox_handle.0).unwrap();
            if image.texture_descriptor.array_layer_count() == 1 {
                image.reinterpret_stacked_2d_as_array(image.height() / image.width());
                image.texture_view_descriptor = Some(TextureViewDescriptor {
                    dimension: Some(TextureViewDimension::Cube),
                    ..default()
                });
            }
            camera.insert(Skybox {
                image: skybox_handle.clone(),
                brightness: 1000.0,
            });
        }
    }
}

fn setup(
    mut commands: Commands,
    settings: Res<settings::SettingsResource>,
    mut window: Query<&mut Window>,
    asset_server: Res<AssetServer>,
) {
    let mut window = window.single_mut();
    window.visible = true;
    let skybox_handle = asset_server.load("skybox/cube.png");
    commands.insert_resource(SkyboxHandle(skybox_handle));
    // spawn a camera to be able to see anything
    let mut camera = commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::new(0., 0.0, 0.), Vec3::Y),

        ..default()
    },));

    if let settings::AntiAliasOption::Taa = settings.anti_alias {
        camera.insert(TAABundle::default());
    };
    commands.insert_resource(AmbientLight {
        color: Color::rgb_u8(234, 212, 165),
        brightness: light_consts::lux::CLEAR_SUNRISE,
    });
    commands.spawn(AudioBundle {
        source: asset_server.load("Neon Heights.mp3"),
        settings: PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: Volume::new(0.2),
            ..default()
        },
    });
    commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                width: Val::Vw(100.0),
                height: Val::Vh(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },

            //background_color: BackgroundColor(Color::WHITE),
            ..default()
        })
        .with_children(|builder| {
            builder.spawn((
                NodeBundle {
                    style: Style {
                        border: UiRect::all(Val::Px(1.)),
                        height: Val::Percent(100.0),
                        max_width: Val::Vw(100.0),
                        aspect_ratio: Some(16.0 / 9.0),
                        ..default()
                    },
                    //background_color: BackgroundColor(Color::RED),
                    //border_color:Color::YELLOW.into(),
                    ..default()
                },
                SafeUi,
            ));
        });
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    MainMenu,
    InGame,
}

fn temp(
    input: Res<ButtonInput<KeyCode>>,
    state: Res<State<InGameState>>,
    mut next_state: ResMut<NextState<InGameState>>,
    mut next_app: ResMut<NextState<AppState>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        match state.get() {
            InGameState::Playing => next_state.set(InGameState::Paused),
            InGameState::Paused => next_state.set(InGameState::Playing),
            InGameState::Upgrade => {}
            InGameState::End => {
                next_state.set(InGameState::None);
                next_app.set(AppState::MainMenu);
            }
            InGameState::None => {}
        }
    }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum InGameState {
    Playing,
    Paused,
    Upgrade,
    End,
    None,
}
