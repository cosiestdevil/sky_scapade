#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use base64::prelude::*;
use bevy::{
    prelude::*,
    window::PresentMode,
    winit::{UpdateMode, WinitSettings},
};
use bevy_ecs::system::EntityCommands;
use bevy_framepace::{FramepaceSettings, Limiter};
use bevy_obj::ObjPlugin;
use bevy_rapier3d::prelude::*;
use bevy_tnua::{
    builtins::{TnuaBuiltinJump, TnuaBuiltinWalk},
    controller::{TnuaController, TnuaControllerBundle, TnuaControllerPlugin},
};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_tnua_rapier3d::{TnuaRapier3dIOBundle, TnuaRapier3dPlugin, TnuaRapier3dSensorShape};
use input::Player;
use iyes_perf_ui::PerfUiPlugin;
use leafwing_input_manager::{
    action_state::ActionState, input_map::InputMap, plugin::InputManagerPlugin, InputManagerBundle,
};
use std::time::Duration;
mod generate;
mod input;
mod menu;
const GAME_NAME: &str = "SkyScapade";
fn main() {
    let mut app = App::new();
    app.add_plugins(EmbeddedAssetPlugin{
        mode:bevy_embedded_assets::PluginMode::ReplaceDefault
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
    
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::default().in_fixed_schedule())
    .add_plugins(RapierDebugRenderPlugin::default())
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
    app.add_plugins(InputManagerPlugin::<input::Action>::default());
    app.add_plugins((
        TnuaControllerPlugin::default(),
        TnuaRapier3dPlugin::default(),
    ));
    app.add_systems(OnExit(AppState::InGame), cleanup_level);
    app.add_systems(OnEnter(AppState::InGame), start_level);
    app.add_systems(Update, (move_player).run_if(in_state(AppState::InGame)));
    app.add_systems(FixedUpdate, (generate_more_if_needed).run_if(in_state(AppState::InGame)));
    app.run();
}

fn generate_more_if_needed(
    mut commands: Commands,
    mut level: Query<(Entity, &mut crate::Level)>,
    player: Query<&Transform, With<Player>>,
    mut generator: ResMut<generate::Generator>,
) {
    let (level_entity, mut level) = level.single_mut();
    let player_transform = player.single();
    if (player_transform.translation.x/20.) >= level.right - 10. {
        for x in 1..256 {
            let x = x+(level.right as usize);
            let hy = (generator.get_height(x) * 10.) as f32;
            commands
                .spawn(Collider::cuboid(10.0, 10., 10.))
                .insert(LevelFloor)
                .insert(TransformBundle::from_transform(Transform::from_xyz(
                    (x as f32) * 20.,
                    hy,
                    0.,
                )))
                .set_parent(level_entity);
            
        }
        level.right+=255.;
    }
}

fn move_player(
    mut query: Query<(&ActionState<input::Action>, &mut TnuaController), With<input::Player>>,
) {
    let (action_state, mut controller) = query.single_mut();
    // Each action has a button-like state of its own that you can check
    //println!("move_player {:?}",action_state);
    let mut direction = Vec3::ZERO;
    if action_state.pressed(&input::Action::Left) {
        direction -= Vec3::X;
    }
    if action_state.pressed(&input::Action::Right) {
        direction += Vec3::X;
    }
    controller.basis(TnuaBuiltinWalk {
        desired_velocity: direction.normalize_or_zero() * 100.0,
        desired_forward: direction.normalize_or_zero(),
        float_height: 15.,
        ..Default::default()
    });
    if action_state.pressed(&input::Action::Jump) {
        controller.action(TnuaBuiltinJump {
            height: 50.,
            ..default()
        });
    }
}

#[derive(Component)]
struct Level {
    right: f32,
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

fn start_level(
    mut commands: Commands,
    mut camera: Query<(Entity, &mut Transform), With<Camera>>,
    safe_ui: Query<Entity, With<crate::SafeUi>>,
) {
    let mut generator = generate::Generator::from_entropy(256., 64., 5);
    let safe_ui = safe_ui.get_single();
    if let Ok(safe_ui) = safe_ui {
        let mut safe_ui = commands.entity(safe_ui);
        safe_ui.with_children(|ui| {
            let seed = BASE64_STANDARD.encode(generator.get_seed());
            ui.spawn(TextBundle::from_section(
                seed,
                TextStyle {
                    color: Color::WHITE,
                    font_size: 24.0,
                    ..default()
                },
            ));
        });
    }
    commands.insert_resource(generator.clone());
    let level = commands
        .spawn((
            Level { right: 255. },
            TransformBundle::default(),
            VisibilityBundle::default(),
        ))
        .id();
    let hy = (generator.get_height(0) * 10.) as f32;
    commands
        .spawn(Collider::cuboid(10.0, 10., 10.))
        .insert(LevelFloor)
        .insert(TransformBundle::from_transform(Transform::from_xyz(
            0., hy, 0.,
        )))
        .set_parent(level);
    let input_map = InputMap::new([
        (input::Action::Jump, KeyCode::Space),
        (input::Action::Left, KeyCode::KeyA),
        (input::Action::Right, KeyCode::KeyD),
    ]);
    let player = commands
        .spawn(Collider::capsule_y(10., 5.))
        .insert(TnuaRapier3dSensorShape(Collider::capsule_y(10., 5.)))
        .insert(TnuaControllerBundle::default())
        .insert(TnuaRapier3dIOBundle::default())
        .insert(input::Player)
        .insert(InputManagerBundle::with_map(input_map))
        .insert(RigidBody::Dynamic)
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(TransformBundle::from(Transform::from_xyz(
            15.,
            hy + 35.,
            0.,
        )))
        .set_parent(level)
        .id();
    if let Ok((camera, mut camera_transform)) = camera.get_single_mut() {
        // =
        commands.entity(camera).set_parent(player);
        *camera_transform =
            Transform::from_xyz(0.0, 50., 200.0).looking_at(Vec3::new(0., 25.0, 0.), Vec3::Y)
    }
    for x in 1..256 {
        let hy = (generator.get_height(x) * 10.) as f32;
        commands
            .spawn(Collider::cuboid(10.0, 10., 10.))
            .insert(LevelFloor)
            .insert(TransformBundle::from_transform(Transform::from_xyz(
                (x as f32) * 20.,
                hy,
                0.,
            )))
            .set_parent(level);
    }
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
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        match state.get() {
            AppState::InGame => next_state.set(AppState::MainMenu),
            AppState::MainMenu => (),
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
