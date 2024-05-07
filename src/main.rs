use std::time::Duration;

use bevy::{
    prelude::*,
    window::PresentMode,
    winit::{UpdateMode, WinitSettings},
};
use bevy_ecs::system::EntityCommands;
use bevy_framepace::{FramepaceSettings, Limiter};
use bevy_obj::ObjPlugin;
use bevy_rapier3d::prelude::*;
use iyes_perf_ui::PerfUiPlugin;
mod generate;
mod menu;

const GAME_NAME: &str = "Cosiest";
fn main() {
    let mut app = App::new();
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
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
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
    app.add_systems(OnEnter(AppState::InGame), (start_level));
    app.run();
}

fn start_level(mut commands: Commands, mut camera: Query<(Entity, &mut Transform), With<Camera>>) {
    let mut generator = generate::Generator::new(1, 256, 64, 5);
    commands.insert_resource(generator.clone());

    let hy = (generator.get_height(0) * 10.) as f32;
    commands
        .spawn(Collider::cuboid(10.0, 10., 10.))
        .insert(TransformBundle::from_transform(Transform::from_xyz(
            0., hy, 0.,
        )));
    let player = commands
        .spawn(Collider::capsule_y(20., 5.))
        .insert(RigidBody::Dynamic)
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(TransformBundle::from(Transform::from_xyz(15., hy+35., 0.))).id();
    if let Ok((camera, mut camera_transform)) = camera.get_single_mut() {
        // =
        commands.entity(camera).set_parent(player);
        *camera_transform=Transform::from_xyz(0.0, 50., 200.0).looking_at(Vec3::new(0., 25.0, 0.), Vec3::Y)
    }
    for x in 1..256 {
        let hy = (generator.get_height(x) * 10.) as f32;
        commands
            .spawn(Collider::cuboid(10.0, 10., 10.))
            .insert(TransformBundle::from_transform(Transform::from_xyz(
                (x as f32) * 20.,
                hy,
                0.,
            )));
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
        return result;
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
