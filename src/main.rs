#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use base64::prelude::*;
use bevy::{
    log,
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    window::PresentMode,
    winit::{UpdateMode, WinitSettings},
};
use bevy_ecs::system::EntityCommands;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_framepace::{FramepaceSettings, Limiter};
use bevy_obj::ObjPlugin;
use bevy_rapier3d::prelude::*;
use bevy_tnua::{
    builtins::{TnuaBuiltinDash, TnuaBuiltinJump, TnuaBuiltinWalk}, control_helpers::TnuaSimpleAirActionsCounter, controller::{TnuaController, TnuaControllerBundle, TnuaControllerPlugin}, TnuaAction
};
use bevy_tnua_rapier3d::{TnuaRapier3dIOBundle, TnuaRapier3dPlugin, TnuaRapier3dSensorShape};
use generate::NoiseSettings;
use input::Player;
use iyes_perf_ui::PerfUiPlugin;
use leafwing_input_manager::{
    action_state::ActionState, input_map::InputMap, plugin::InputManagerPlugin, InputManagerBundle,
};
use std::time::Duration;
use strum::EnumIter;
mod generate;
mod input;
mod menu;
mod upgrades;
const GAME_NAME: &str = "SkyScapade";
fn main() {
    let mut app = App::new();
    app.add_plugins(EmbeddedAssetPlugin {
        mode: bevy_embedded_assets::PluginMode::ReplaceDefault,
    });
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: GAME_NAME.into(),
                    //resolution: (2560.0, 1080.0).into(),
                    resolution: (1280., 720.).into(),
                    name: Some("new_game_1.app".into()),
                    present_mode: PresentMode::Mailbox,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    )
    .insert_resource(RapierConfiguration {
        // gravity: Vec2::ZERO,
        timestep_mode: TimestepMode::Fixed {
            dt: 1.0 / 64.0,
            substeps: 1,
        },
        ..RapierConfiguration::new(1.0)
    })
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::default().in_fixed_schedule())
    //.add_plugins(RapierDebugRenderPlugin::default())
    .insert_resource(WinitSettings {
        focused_mode: UpdateMode::Continuous,
        unfocused_mode: UpdateMode::ReactiveLowPower {
            wait: Duration::from_secs_f64(1.0 / 30.0), //Duration::MAX
        },
    })
    .add_plugins(menu::MenuPlugin)
    .add_plugins(ObjPlugin)
    .insert_state(AppState::MainMenu)
    //.add_plugins(ScreenDiagnosticsPlugin::default())
    //.add_plugins(ScreenFrameDiagnosticsPlugin)
    .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
    .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
    //.add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
    //.add_plugins(system_info::SystemInformationDiagnosticsPlugin)
    .add_plugins(bevy_framepace::FramepacePlugin)
    .add_plugins(PerfUiPlugin)
    .add_systems(Startup, setup)
    .add_systems(Update, temp);
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
        (move_player, move_camera_based_on_speed)
            .run_if(in_state(AppState::InGame).and_then(in_state(InGameState::Playing))),
    );
    app.add_systems(
        FixedUpdate,
        (
            generate_more_if_needed,
            level_upgrade,
            level_finish,
            killing_floor,
            update_score,
            dash_cooldown,
        )
            .run_if(in_state(AppState::InGame).and_then(in_state(InGameState::Playing))),
    );
    app.add_systems(OnEnter(InGameState::Paused), pause_level);
    app.add_systems(OnExit(InGameState::Paused), resume_level);
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
    mut camera: Query<&mut Transform, With<Camera>>,
    velocities: Query<&Velocity, With<Player>>,
) {
    let cube_size = 1.0f32;
    let mut camera = camera.single_mut();
    let player_velocity = velocities.single();

    camera.translation.z = (cube_size * 20.) + (player_velocity.linvel.x.abs().sqrt() - 5.).max(0.);
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

    match time_text.get_single_mut() {
        Ok(mut time_text) => {
            let time = level.timer.elapsed().as_secs();
            time_text.sections[0].value = format!("Time: {:02}:{:02}", time / 60, time % 60);
        }
        Err(_) => {}
    }
    if level.timer.just_finished() {
        log::info!("Level Finished. Travelled: {}", player.translation.x);
        next_state.set(InGameState::End);
    }
}
fn level_upgrade(
    //mut commands: Commands,
    time: Res<Time>,
    mut level: Query<&mut Level>,
    mut player: Query<&mut Player>,
    mut generator: ResMut<generate::Generator>,
) {
    let mut level = level.single_mut();
    level.upgrade_timer.tick(time.delta());
    if level.upgrade_timer.just_finished() {
        let upgrade = generator.get_upgrade();
        log::info!("Upgrade:{:?}", upgrade);
        if let Some(upgrade) = upgrade {
            let mut player = player.single_mut();
            match upgrade {
                UpgradeType::Speed(upgrade) => {
                    player.speed_modifiers.push(upgrade);
                }
                UpgradeType::JumpPower(upgrade) => {
                    player.jump_modifiers.push(upgrade);
                }
                UpgradeType::JumpSkill(skill) => {
                    player.jump_skill = skill;
                }
                UpgradeType::DashSkill(skill) => {
                    player.dash_skill = skill;
                }
                _ => {}
            }
        }
    }
}
#[derive(Debug, Copy, Clone)]
enum UpgradeType {
    Speed(StatUpgrade),
    JumpPower(StatUpgrade),
    JumpSkill(JumpSkill),
    DashSkill(DashSkill),
}
#[derive(EnumIter, Debug, PartialEq, Copy, Clone, PartialOrd)]
enum UpgradeLevel {
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
#[derive(Debug, Copy, Clone)]
struct JumpSkill {
    max_jumps: u8,
    tier: UpgradeLevel,
    air: bool,
}
#[derive(Debug, Copy, Clone)]
struct DashSkill {
    max_dash: u8,
    air: bool,
    cooldown: Duration,
    tier: UpgradeLevel,
}
#[derive(Debug, Copy, Clone)]
struct StatUpgrade {
    modifier: f32,
    additive: bool,
    tier: UpgradeLevel,
}

impl upgrades::Upgrade<UpgradeType> for UpgradeType {
    fn is_lower(&self, other: UpgradeType) -> bool {
        match self {
            UpgradeType::Speed(me) => {
                return match other {
                    UpgradeType::Speed(other) => me.tier <= other.tier,
                    _ => false,
                }
            }
            UpgradeType::JumpPower(me) => {
                return match other {
                    UpgradeType::JumpPower(other) => me.tier <= other.tier,
                    _ => false,
                }
            }
            UpgradeType::JumpSkill(me) => {
                return match other {
                    UpgradeType::JumpSkill(other) => me.tier <= other.tier,
                    _ => false,
                }
            }
            UpgradeType::DashSkill(me) => {
                return match other {
                    UpgradeType::DashSkill(other) => me.tier <= other.tier,
                    _ => false,
                }
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
    match score.get_single_mut() {
        Ok(mut score_text) => {
            score_text.sections[0].value = format!("Score: {}", player.score);
        }
        Err(_) => {}
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
    if (player_transform.translation.x / (cube_size * 2.)) >= level.right - 10. {
        let mut hole_streak = 0;
        for x in 0..256 {
            let x = x + (level.right as usize);
            let hy = (generator.get_height(x) as f32) * cube_size;
            let platform_assets = platform_assets.clone();
            if hole_streak > 4 {
                hole_streak = 0;
            } else {
                if generator.is_hole(x) {
                    hole_streak += 1;
                    continue;
                }
            }
            commands
                .spawn(Collider::cuboid(cube_size, cube_size, cube_size))
                .insert(PbrBundle {
                    mesh: platform_assets.mesh.clone(),
                    material: platform_assets.material.clone(),
                    ..default()
                })
                .insert(LevelFloor)
                .insert(TransformBundle::from_transform(Transform::from_xyz(
                    (x as f32) * cube_size * 2.,
                    hy,
                    0.,
                )))
                .set_parent(level_entity);
        }
        level.right += 255.;
    }
}
fn dash_cooldown(mut player: Query<&mut Player>, time: Res<Time>) {
    let mut player = player.single_mut();
    if let Some(ref mut cooldown) = player.dash_cooldown {
        cooldown.tick(time.delta());
        if cooldown.just_finished() {
            player.used_dashes -= 1;
            //info!("Dash Cooldown");
            info!("Used Dashes: {}", player.used_dashes);
            if player.used_dashes == 0 {
                player.dash_cooldown = None;
            } else {
                player.dash_cooldown =
                    Some(Timer::new(player.dash_skill.cooldown, TimerMode::Once));
            }
        }
    }
}
fn move_player(
    mut query: Query<(
        &ActionState<input::Action>,
        &mut TnuaController,
        &mut input::Player,
        &mut TnuaSimpleAirActionsCounter,
    )>,
) {
    let (action_state, mut controller, mut player,mut air_actions_counter) = query.single_mut();
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
    controller.basis(TnuaBuiltinWalk {
        desired_velocity: direction.normalize_or_zero() * player.max_speed(),
        desired_forward: direction.normalize_or_zero(),
        float_height: 2.,
        ..Default::default()
    });
    if action_state.just_pressed(&input::Action::Dash) {
        if player.dash_skill.max_dash > player.used_dashes {
            if let None = player.dash_cooldown {
                player.dash_cooldown =
                    Some(Timer::new(player.dash_skill.cooldown, TimerMode::Once));
            }

            player.used_dashes += 1;
            info!("Used Dashes: {}", player.used_dashes);
            controller.action(TnuaBuiltinDash {
                displacement: direction.normalize_or_zero() * player.max_speed() * 0.75,
                speed: player.max_speed() * 3.,
                allow_in_air: player.dash_skill.air,
                ..default()
            });
        }
    }

    if action_state.pressed(&input::Action::Jump) {
        let air_jumps:usize = (player.jump_skill.max_jumps - 1).into();
        controller.action(TnuaBuiltinJump {
            height: player.jump_power(),
            allow_in_air: player.jump_skill.air && air_actions_counter.air_count_for(TnuaBuiltinJump::NAME)
            <= air_jumps,
            ..default()
        });
    }
}

#[derive(Component)]
struct Level {
    right: f32,
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
    //mut commands: Commands,
    player: Query<(Entity, &Transform), With<Player>>,
    mut next_state: ResMut<NextState<InGameState>>,
) {
    let (_entity, player) = player.single();
    if player.translation.y < -10. {
        next_state.set(InGameState::End);
    }
}

fn start_level(
    mut commands: Commands,
    mut camera: Query<(Entity, &mut Transform), With<Camera>>,
    safe_ui: Query<Entity, With<crate::SafeUi>>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut next_state: ResMut<NextState<InGameState>>,
) {
    next_state.set(InGameState::Playing);
    let mut generator = generate::Generator::from_entropy(
        NoiseSettings::new(256, 64, 5),
        NoiseSettings::new(7, 64, 7),
        vec![],
    );
    let platform_mesh: Handle<Mesh> = asset_server.load("platform.obj");
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    let platform_assets = PlatformAssets {
        mesh: platform_mesh.clone(),
        material: debug_material.clone(),
    };
    commands.insert_resource(platform_assets);
    let safe_ui = safe_ui.get_single();
    if let Ok(safe_ui) = safe_ui {
        let mut safe_ui = commands.entity(safe_ui);
        safe_ui.with_children(|ui| {
            let seed = BASE64_STANDARD.encode(generator.get_seed());
            ui.spawn(NodeBundle {
                style: Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
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
            });
        });
    }
    commands.insert_resource(generator.clone());
    let level = commands
        .spawn((
            Level {
                right: 255.,
                upgrade_timer: Timer::new(Duration::from_secs(10), TimerMode::Repeating),
                timer: Timer::new(Duration::from_secs(300), TimerMode::Once),
            },
            TransformBundle::default(),
            VisibilityBundle::default(),
        ))
        .id();
    let cube_size = 1.0f32;
    let hy = (generator.get_height(0) as f32) * cube_size;
    commands
        .spawn(Collider::cuboid(cube_size, cube_size, cube_size))
        .insert(PbrBundle {
            mesh: platform_mesh.clone(),
            material: debug_material.clone(),
            ..default()
        })
        .insert(LevelFloor)
        .insert(TransformBundle::from_transform(Transform::from_xyz(
            0., hy, 0.,
        )))
        .set_parent(level);
    let input_map = InputMap::new([
        (input::Action::Jump, KeyCode::Space),
        (input::Action::Left, KeyCode::KeyA),
        (input::Action::Right, KeyCode::KeyD),
        (input::Action::Dash, KeyCode::ShiftLeft),
    ]);
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
            dash_skill: DashSkill {
                max_dash: 0,
                tier: UpgradeLevel::None,
                air: false,
                cooldown: Duration::from_secs(10),
            },
            dash_cooldown: None,
            used_dashes: 0,
            score: 0.0,
        })
        .insert(InputManagerBundle::with_map(input_map))
        .insert(TnuaSimpleAirActionsCounter::default())
        .insert(RigidBody::Dynamic)
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(TransformBundle::from(Transform::from_xyz(
            1.5 * cube_size,
            hy + (3.5 * cube_size),
            0.,
        )))
        .set_parent(level)
        .with_children(|player| {
            player.spawn(PointLightBundle {
                point_light: PointLight {
                    shadows_enabled: true,
                    intensity: 100_000_000.,
                    range: 1000.0,
                    ..default()
                },
                transform: Transform::from_xyz(10.0, 60.0, 10.0),
                ..default()
            });
        })
        .id();
    if let Ok((camera, mut camera_transform)) = camera.get_single_mut() {
        // =
        commands.entity(camera).set_parent(player);
        *camera_transform = Transform::from_xyz(0.0, cube_size * 5., cube_size * 20.)
            .looking_at(Vec3::new(0., 0., 0.), Vec3::Y)
    }
    for x in 1..6 {
        let hy = (generator.get_height(x) as f32) * cube_size;
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
    for x in 6..256 {
        let hy = (generator.get_height(x) as f32) * cube_size;
        if hole_streak > 4 {
            hole_streak = 0;
        } else {
            if generator.is_hole(x) {
                hole_streak += 1;
                continue;
            }
        }
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
            button.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    color: Color::RED,
                    font_size: 22.0,
                    ..default()
                },
            ));
        });
        result
    }
}

fn setup(mut commands: Commands, mut frame_pace_settings: ResMut<FramepaceSettings>) {
    frame_pace_settings.limiter = Limiter::Off;
    // spawn a camera to be able to see anything
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::new(0., 0.0, 0.), Vec3::Y),
        ..default()
    });

    // create a simple Perf UI with default settings
    // and all entries provided by the crate:
    // commands.spawn((
    //     PerfUiRoot::default(),
    //     (
    //         PerfUiEntryFPS::default(),
    //         //PerfUiEntryFPSWorst::default(),
    //         PerfUiEntryFrameTime::default(),
    //         //PerfUiEntryFrameTimeWorst::default(),
    //         //PerfUiEntryFrameCount::default(),
    //         PerfUiEntryEntityCount::default(),
    //     ),
    //     // (
    //     //     PerfUiEntryCpuUsage::default(),
    //     //     PerfUiEntryMemUsage::default(),
    //     // ),
    //     (
    //         //PerfUiEntryFixedTimeStep::default(),
    //         //PerfUiEntryFixedOverstep::default(),
    //         //PerfUiEntryRunningTime::default(),
    //         PerfUiEntryClock::default(),
    //     ),
    //     (
    //         //PerfUiEntryCursorPosition::default(),
    //         PerfUiEntryWindowResolution::default(),
    //         PerfUiEntryWindowScaleFactor::default(),
    //         PerfUiEntryWindowMode::default(),
    //         PerfUiEntryWindowPresentMode::default(),
    //     ),
    //));
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
                        height: Val::Percent(100.0),
                        max_width: Val::Vw(100.0),
                        aspect_ratio: Some(16.0 / 9.0),
                        ..default()
                    },
                    //background_color: BackgroundColor(Color::RED),
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
            InGameState::End => next_app.set(AppState::MainMenu),
            InGameState::None => {}
        }
    }
}

mod system_info {
    use bevy::prelude::*;
    use bevy_ecs::{prelude::ResMut, system::Local};
    use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

    use bevy_diagnostic::{Diagnostic, DiagnosticPath, Diagnostics, DiagnosticsStore};

    const BYTES_TO_GIB: f64 = 1.0 / 1024.0 / 1024.0 / 1024.0;

    #[derive(Default)]
    pub struct SystemInformationDiagnosticsPlugin;
    impl Plugin for SystemInformationDiagnosticsPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(Startup, setup_system)
                .add_systems(FixedUpdate, diagnostic_system);
        }
    }

    impl SystemInformationDiagnosticsPlugin {
        pub const CPU_USAGE: DiagnosticPath = DiagnosticPath::const_new("system/cpu_usage");
        pub const MEM_USAGE: DiagnosticPath = DiagnosticPath::const_new("system/mem_usage");
    }

    pub(crate) fn setup_system(mut diagnostics: ResMut<DiagnosticsStore>) {
        diagnostics
            .add(Diagnostic::new(SystemInformationDiagnosticsPlugin::CPU_USAGE).with_suffix("%"));
        diagnostics
            .add(Diagnostic::new(SystemInformationDiagnosticsPlugin::MEM_USAGE).with_suffix("%"));
    }

    pub(crate) fn diagnostic_system(
        mut diagnostics: Diagnostics,
        mut sysinfo: Local<Option<System>>,
    ) {
        if sysinfo.is_none() {
            *sysinfo = Some(System::new_with_specifics(
                RefreshKind::new()
                    .with_cpu(CpuRefreshKind::new().with_cpu_usage())
                    .with_memory(MemoryRefreshKind::everything()),
            ));
        }
        let Some(sys) = sysinfo.as_mut() else {
            return;
        };

        sys.refresh_cpu_specifics(CpuRefreshKind::new().with_cpu_usage());
        sys.refresh_memory();
        let current_cpu_usage = sys.global_cpu_info().cpu_usage();
        // `memory()` fns return a value in bytes
        let total_mem = sys.total_memory() as f64 / BYTES_TO_GIB;
        let used_mem = sys.used_memory() as f64 / BYTES_TO_GIB;
        let current_used_mem = used_mem / total_mem * 100.0;

        diagnostics.add_measurement(&SystemInformationDiagnosticsPlugin::CPU_USAGE, || {
            current_cpu_usage as f64
        });
        diagnostics.add_measurement(&SystemInformationDiagnosticsPlugin::MEM_USAGE, || {
            current_used_mem
        });
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
