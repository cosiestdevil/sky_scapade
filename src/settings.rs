use std::{fs, io::Read};

use bevy::{prelude::*, window::WindowMode};
use bevy_framepace::{FramepaceSettings, Limiter};
use bevy_persistent::{Persistent, StorageFormat};
use serde::{Deserialize, Serialize};
pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        let config_dir = dirs::config_dir().unwrap().join(env!("CARGO_PKG_NAME"));

        if let Ok(mut settings_file) = fs::File::open(config_dir.join("settings.toml")) {
            let mut buf: String = Default::default();
            let _ = settings_file.read_to_string(&mut buf);
            if let Ok(settings) = toml::from_str::<Settings>(buf.as_str()) {
                match settings.anti_alias {
                    AntiAliasOption::Taa => {
                        app.add_plugins(
                            bevy::core_pipeline::experimental::taa::TemporalAntiAliasPlugin,
                        );
                    }
                    msaa if msaa.is_msaa() => {
                        app.insert_resource(msaa.msaa_resource().unwrap());
                    }
                    _=>{}
                }
            }
        }

        app.insert_resource(
            SettingsResource::builder()
                .name("settings")
                .format(StorageFormat::Toml)
                .path(config_dir.join("settings.toml"))
                .default(Settings::default())
                .build()
                .expect("Failed to load settings"),
        );
        app.add_systems(Startup, settings_changed);
        app.add_systems(FixedUpdate, settings_changed);
    }
}
pub type SettingsResource = Persistent<Settings>;
#[derive(Resource, Serialize, Deserialize, Default)]
pub struct Settings {
    #[serde(default)]
    pub frame_limit: FrameLimitOption,
    #[serde(default)]
    pub window_mode: WindowModeOption,
    #[serde(default)]
    pub anti_alias: AntiAliasOption,
}


pub trait SettingsCycleOption{
    fn next(&self) -> Self;
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
}
impl SettingsCycleOption for FrameLimitOption{
    
    fn next(&self) -> Self {
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
}
impl SettingsCycleOption for WindowModeOption{
    fn next(&self) -> Self {
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

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
pub enum AntiAliasOption {
    Off,
    Msaa2,
    #[default]
    Msaa4,
    Msaa8,
    Taa,
}

impl AntiAliasOption {
    pub fn label(&self)-> &'static str{
        match self{
            AntiAliasOption::Off => "Off",
            AntiAliasOption::Msaa2 => "MSAA (x2)",
            AntiAliasOption::Msaa4 => "MSAA (X4)",
            AntiAliasOption::Msaa8 => "MSAA (X8)",
            AntiAliasOption::Taa => "TAA",
        }
    }

    pub fn msaa_resource(&self) -> Option<Msaa> {
        match self {
            AntiAliasOption::Off => Some(Msaa::Off),
            AntiAliasOption::Msaa2 => Some(Msaa::Sample2),
            AntiAliasOption::Msaa4 => Some(Msaa::Sample4),
            AntiAliasOption::Msaa8 => Some(Msaa::Sample8),
            AntiAliasOption::Taa => None,
        }
    }
    pub fn is_msaa(&self) -> bool {
        matches!(self, AntiAliasOption::Off
            | AntiAliasOption::Msaa2
            | AntiAliasOption::Msaa4
            | AntiAliasOption::Msaa8)
    }
}

impl SettingsCycleOption for AntiAliasOption{
    fn next(&self) -> Self {
        match self{
            AntiAliasOption::Off => AntiAliasOption::Msaa2,
            AntiAliasOption::Msaa2 => AntiAliasOption::Msaa4,
            AntiAliasOption::Msaa4 => AntiAliasOption::Msaa8,
            AntiAliasOption::Msaa8 => AntiAliasOption::Taa,
            AntiAliasOption::Taa =>  AntiAliasOption::Off,
        }
    }
}

fn settings_changed(
    mut windows: Query<&mut Window>,
    mut frame_pace_settings: ResMut<FramepaceSettings>,
    settings: Res<crate::settings::SettingsResource>,
) {
    if settings.is_changed() {
        frame_pace_settings.limiter = settings.frame_limit.into();
        for mut window in &mut windows {
            window.mode = settings.window_mode.into();
        }
    }
}
