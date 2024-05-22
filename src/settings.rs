use bevy::{prelude::*, window::WindowMode};
use bevy_framepace::{FramepaceSettings, Limiter};
use bevy_persistent::{Persistent, StorageFormat};
use serde::{Deserialize, Serialize};
pub struct SettingsPlugin;
impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        let config_dir = dirs::config_dir().unwrap().join(env!("CARGO_PKG_NAME"));
        app.insert_resource(
            SettingsResource::builder()
                .name("settings")
                .format(StorageFormat::Toml)
                .path(config_dir.join("settings.toml"))
                .default(Settings {
                    frame_limit: FrameLimitOption::Off,
                    window_mode: WindowModeOption::Windowed,
                })
                .build()
                .expect("Failed to load settings"),
        );
        app.add_systems(Startup, setup);
    }
}
pub type SettingsResource = Persistent::<Settings>;
#[derive(Resource, Serialize, Deserialize, Default)]
pub struct Settings {
    #[serde(default)]
    pub frame_limit: FrameLimitOption,
    #[serde(default)]
    pub window_mode: WindowModeOption,
}
#[derive(Serialize, Deserialize, Default, Clone, Copy)]
pub enum FrameLimitOption {
    #[default]
    Off,
    Cinematic,
    Standard,
    High,
}
impl FrameLimitOption {
    pub fn label(&self) -> &'static str {
        match self {
            FrameLimitOption::Off => "Infinite",
            FrameLimitOption::Cinematic => "Cinematic",
            FrameLimitOption::Standard => "Standard",
            FrameLimitOption::High => "High",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            FrameLimitOption::Off => Self::Cinematic,
            FrameLimitOption::Cinematic => Self::Standard,
            FrameLimitOption::Standard => Self::High,
            FrameLimitOption::High => Self::Off,
        }
    }
}
impl From<FrameLimitOption> for Limiter {
    fn from(value: FrameLimitOption) -> Self {
        match value {
            FrameLimitOption::Off => Limiter::Off,
            FrameLimitOption::Cinematic => Limiter::from_framerate(30.0),
            FrameLimitOption::Standard => Limiter::from_framerate(60.0),
            FrameLimitOption::High => Limiter::from_framerate(120.0),
        }
    }
}
#[derive(Serialize, Deserialize, Default, Clone, Copy)]
pub enum WindowModeOption {
    #[default]
    Windowed,
    BorderlessFullscreen,
    Fullscreen,
}
impl WindowModeOption {
    pub fn label(&self) -> &'static str {
        match self {
            WindowModeOption::Windowed => "Windowed",
            WindowModeOption::BorderlessFullscreen => "Borderless",
            WindowModeOption::Fullscreen => "Fullscreen",
        }
    }
    pub fn next(&self) -> Self {
        match self {
            WindowModeOption::Windowed => WindowModeOption::BorderlessFullscreen,
            WindowModeOption::BorderlessFullscreen => WindowModeOption::Fullscreen,
            WindowModeOption::Fullscreen => WindowModeOption::Windowed,
        }
    }
}
impl From<WindowModeOption> for WindowMode {
    fn from(value: WindowModeOption) -> Self {
        match value {
            WindowModeOption::Windowed => WindowMode::Windowed,
            WindowModeOption::BorderlessFullscreen => WindowMode::BorderlessFullscreen,
            WindowModeOption::Fullscreen => WindowMode::Fullscreen,
        }
    }
}
fn setup(
    settings: Res<SettingsResource>,
    mut window: Query<&mut Window>,
    mut frame_pace_settings: ResMut<FramepaceSettings>,
) {
    frame_pace_settings.limiter = settings.frame_limit.into();
    let mut window = window.single_mut();
    window.mode = settings.window_mode.into();
}
