use std::f32::consts::PI;

use crate::UiHelper;
use bevy::{
    prelude::*,
    render::{
        render_resource::{encase::vector::FromVectorParts, Face},
        texture::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    },
};
use bevy_framepace::{FramepaceSettings, Limiter};
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(MainMenuState::Menu);
        app.insert_resource(FrameLimitResource(FrameLimitOption::Off));
        app.add_systems(Startup,setup);
        app.add_systems(
            Update,
            (main_menu_button_system, settings_menu_button_system),
        );
        app.add_systems(OnEnter(crate::AppState::MainMenu), enter_main_menu);
        app.add_systems(OnExit(crate::AppState::MainMenu), exit_main_menu);
        app.add_systems(
            FixedUpdate,
            (main_menu_fixed_update).run_if(in_state(crate::AppState::MainMenu)),
        );
        app.add_systems(OnEnter(MainMenuState::Settings), enter_settings);
        app.add_systems(OnExit(MainMenuState::Settings), exit_settings);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let menu_back_image: Handle<Image> =
        asset_server.load_with_settings("cyberpunk_back.png", |s: &mut ImageLoaderSettings| {
            match &mut s.sampler {
                ImageSampler::Default => {
                    s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                        address_mode_u: ImageAddressMode::Repeat,
                        address_mode_v: ImageAddressMode::Repeat,
                        ..default()
                    });
                }
                ImageSampler::Descriptor(sampler) => {
                    sampler.address_mode_u = ImageAddressMode::Repeat;
                    sampler.address_mode_v = ImageAddressMode::Repeat;
                }
            }
        });
    let background_material = materials.add(StandardMaterial {
        base_color_texture: Some(menu_back_image.clone()),
        cull_mode: Some(Face::Front),
        double_sided: true,
        unlit: true,
        ..default()
    });
    let menu_middle_image: Handle<Image> =
        asset_server.load_with_settings("cyberpunk_middle.png", |s: &mut ImageLoaderSettings| {
            match &mut s.sampler {
                ImageSampler::Default => {
                    s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                        address_mode_u: ImageAddressMode::Repeat,
                        address_mode_v: ImageAddressMode::Repeat,
                        ..default()
                    });
                }
                ImageSampler::Descriptor(sampler) => {
                    sampler.address_mode_u = ImageAddressMode::Repeat;
                    sampler.address_mode_v = ImageAddressMode::Repeat;
                }
            }
        });
    let middle_material = materials.add(StandardMaterial {
        base_color_texture: Some(menu_middle_image.clone()),
        cull_mode: Some(Face::Front),
        alpha_mode: AlphaMode::Mask(0.0),
        double_sided: true,
        unlit: true,
        ..default()
    });
    let menu_front_image: Handle<Image> =
        asset_server.load_with_settings("cyberpunk_front.png", |s: &mut ImageLoaderSettings| {
            match &mut s.sampler {
                ImageSampler::Default => {
                    s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                        address_mode_u: ImageAddressMode::Repeat,
                        address_mode_v: ImageAddressMode::Repeat,
                        ..default()
                    });
                }
                ImageSampler::Descriptor(sampler) => {
                    sampler.address_mode_u = ImageAddressMode::Repeat;
                    sampler.address_mode_v = ImageAddressMode::Repeat;
                }
            }
        });
    let front_material = materials.add(StandardMaterial {
        base_color_texture: Some(menu_front_image.clone()),
        cull_mode: Some(Face::Front),
        alpha_mode: AlphaMode::Mask(0.0),
        double_sided: true,
        unlit: true,
        ..default()
    });
    let cylinder: Handle<Mesh> = asset_server.load("hollow_cylinder.obj");
    commands
        .spawn((
            MenuBackground,
            TransformBundle::default(),
            VisibilityBundle::default(),
        ))
        .with_children(|menu_background| {
            menu_background.spawn(PointLightBundle {
                point_light: PointLight {
                    shadows_enabled: true,
                    intensity: 10_000_000.,
                    range: 100.0,
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 6.0, 6.0),
                ..default()
            });
            menu_background.spawn((
                PbrBundle {
                    mesh: cylinder.clone(),
                    material: background_material.clone(),
                    transform: Transform::from_xyz(0.0, 1.0, 0.0)
                        //.with_rotation(Quat::from_rotation_x(PI))
                        .with_scale(Vec3::from_parts([25.0, 10.0, 25.0])),
                    ..default()
                },
                BackgroundLayerComponent(BackgroundLayer::Background),
            ));
            menu_background.spawn((
                PbrBundle {
                    mesh: cylinder.clone(),
                    material: middle_material.clone(),
                    transform: Transform::from_xyz(0.0, 1.0, 1.0)
                        //.with_rotation(Quat::from_rotation_x(PI))
                        .with_scale(Vec3::from_parts([18.0, 8.0, 18.0])),
                    ..default()
                },
                BackgroundLayerComponent(BackgroundLayer::BackMiddle),
            ));
            menu_background.spawn((
                PbrBundle {
                    mesh: cylinder.clone(),
                    material: middle_material.clone(),
                    transform: Transform::from_xyz(0.0, 0.0, 0.0)
                        .with_rotation(Quat::from_rotation_y(PI / 16.))
                        .with_scale(Vec3::from_parts([12.0, 5.0, 12.0])),
                    ..default()
                },
                BackgroundLayerComponent(BackgroundLayer::Middle),
            ));
            menu_background.spawn((
                PbrBundle {
                    mesh: cylinder.clone(),
                    material: front_material.clone(),
                    transform: Transform::from_xyz(0.0, 0.0, 0.0)
                        //.with_rotation(Quat::from_rotation_x(PI))
                        .with_scale(Vec3::from_parts([6.0, 2.5, 6.0])),
                    ..default()
                },
                BackgroundLayerComponent(BackgroundLayer::Foreground),
            ));
        });
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MainMenuState {
    Menu,
    NewGame,
    Settings,
}
enum MainMenuButton {
    NewGame,
    Settings,
}
#[derive(Component)]
struct MainMenuButtonComponent(MainMenuButton);

fn main_menu_fixed_update(mut camera_query: Query<(&mut Transform, &BackgroundLayerComponent)>) {
    for (mut transform, layer) in camera_query.iter_mut() {
        let base_angle = PI / 5760.;
        match layer.0 {
            BackgroundLayer::Background => transform.rotate_axis(Vec3::Y, base_angle),
            BackgroundLayer::BackMiddle => transform.rotate_axis(Vec3::Y, base_angle * 1.3),
            BackgroundLayer::Middle => transform.rotate_axis(Vec3::Y, base_angle * 1.5),
            BackgroundLayer::Foreground => transform.rotate_axis(Vec3::Y, base_angle * 2.),
        };
    }
    //let mut transform = camera_query.get_single_mut().unwrap();
}
fn settings_menu_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
            &SettingsMenuButtonComponent,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut frame_limit: ResMut<FrameLimitResource>,
    mut frame_pace_settings: ResMut<FramepaceSettings>,
    mut next_menu: ResMut<NextState<MainMenuState>>,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut color, mut border_color, children, button) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                *color = Color::BLACK.into();
                border_color.0 = Color::RED;
                match button.0 {
                    SettingsMenuButton::Apply => next_menu.set(MainMenuState::Menu),
                    SettingsMenuButton::FrameLimit => {
                        match frame_limit.0 {
                            FrameLimitOption::Off => {
                                frame_limit.0 = FrameLimitOption::Cinematic;
                                frame_pace_settings.limiter = Limiter::from_framerate(30.0);
                                text.sections[0].value = "Cinematic".into();
                            }
                            FrameLimitOption::Cinematic => {
                                frame_limit.0 = FrameLimitOption::Standard;
                                frame_pace_settings.limiter = Limiter::from_framerate(60.0);
                                text.sections[0].value = "Standard".into();
                            }
                            FrameLimitOption::Standard => {
                                frame_limit.0 = FrameLimitOption::High;
                                frame_pace_settings.limiter = Limiter::from_framerate(120.0);
                                text.sections[0].value = "High".into();
                            }
                            FrameLimitOption::High => {
                                frame_limit.0 = FrameLimitOption::Off;
                                frame_pace_settings.limiter = Limiter::Off;
                                text.sections[0].value = "Off".into();
                            }
                        };
                    }
                }
            }
            Interaction::Hovered => {
                *color = Color::RED.into();
                border_color.0 = Color::WHITE;
                text.sections[0].style.color = Color::WHITE;
            }
            Interaction::None => {
                *color = Color::WHITE.into();
                border_color.0 = Color::RED;
                text.sections[0].style.color = Color::RED;
            }
        }
    }
}
#[derive(Component)]
struct MenuBackground;

enum FrameLimitOption {
    Off,
    Cinematic,
    Standard,
    High,
}

#[derive(Resource)]
struct FrameLimitResource(FrameLimitOption);

enum BackgroundLayer {
    Background,
    BackMiddle,
    Middle,
    Foreground,
}
#[derive(Component)]
struct BackgroundLayerComponent(BackgroundLayer);

fn main_menu_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
            &MainMenuButtonComponent,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<crate::AppState>>,
    mut next_menu: ResMut<NextState<MainMenuState>>,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut color, mut border_color, children, button) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                *color = Color::BLACK.into();
                border_color.0 = Color::RED;
                match button.0 {
                    MainMenuButton::NewGame => next_state.set(crate::AppState::InGame),
                    MainMenuButton::Settings => next_menu.set(MainMenuState::Settings),
                }
            }
            Interaction::Hovered => {
                *color = Color::RED.into();
                border_color.0 = Color::WHITE;
                text.sections[0].style.color = Color::WHITE;
            }
            Interaction::None => {
                *color = Color::WHITE.into();
                border_color.0 = Color::RED;
                text.sections[0].style.color = Color::RED;
            }
        }
    }
}

fn get_main_menu_menu_bundle() -> NodeBundle {
    NodeBundle {
        style: Style {
            display: Display::Grid,
            grid_template_columns: vec![GridTrack::fr(1.0)],
            grid_template_rows: vec![
                GridTrack::px(100.0),
                RepeatedGridTrack::px(GridTrackRepetition::AutoFill, 32.0),
            ],
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(10.0)),
            row_gap: Val::Px(10.0),
            column_gap: Val::Px(10.0),
            ..default()
        },
        ..default()
    }
}
fn enter_main_menu(safe_ui: Query<Entity, With<crate::SafeUi>>, mut commands: Commands) {
    let safe_ui = safe_ui.get_single();
    if let Ok(safe_ui) = safe_ui {
        let mut safe_ui = commands.entity(safe_ui);
        safe_ui.with_children(|builder| {
            builder
                .spawn((
                    NodeBundle {
                        style: Style {
                            display: Display::Grid,
                            grid_template_columns: vec![
                                GridTrack::auto(),
                                GridTrack::auto(),
                                GridTrack::fr(1.0),
                            ],
                            grid_template_rows: vec![GridTrack::fr(1.0)],
                            //grid_template_rows:vec![GridTrack::px(75.0),RepeatedGridTrack::px(GridTrackRepetition::AutoFill,32.0)],
                            ..default()
                        },
                        ..default()
                    },
                    MainMenu,
                ))
                .with_children(|menu_base| {
                    menu_base
                        .spawn(get_main_menu_menu_bundle())
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                crate::GAME_NAME,
                                TextStyle {
                                    color: Color::WHITE,
                                    font_size: 84.0,
                                    ..default()
                                },
                            ));
                            parent.new_menu_button(
                                "New Game",
                                MainMenuButtonComponent(MainMenuButton::NewGame),
                            );
                            parent.new_menu_button(
                                "Settings",
                                MainMenuButtonComponent(MainMenuButton::Settings),
                            );
                        });
                });
        });
    }
}
fn exit_main_menu(main_menu: Query<Entity, With<MainMenu>>, mut commands: Commands) {
    let main_menu = main_menu.get_single();
    if let Ok(main_menu) = main_menu {
        commands.entity(main_menu).despawn_recursive();
    }
}

fn enter_settings(main_menu: Query<Entity, With<MainMenu>>, mut commands: Commands) {
    let main_menu = main_menu.get_single();
    if let Ok(main_menu) = main_menu {
        let mut main_menu = commands.entity(main_menu);
        main_menu.with_children(|menu_base| {
            menu_base
                .spawn((SettingsMenu, get_main_menu_menu_bundle()))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Settings",
                        TextStyle {
                            color: Color::WHITE,
                            font_size: 42.0,
                            ..default()
                        },
                    ));
                    parent
                        .spawn(NodeBundle::default())
                        .with_children(|frame_rate| {
                            frame_rate.spawn(TextBundle::from_section(
                                "Frame Rate: ",
                                TextStyle {
                                    color: Color::WHITE,
                                    font_size: 42.0,
                                    ..default()
                                },
                            ));
                            frame_rate.new_menu_button(
                                "Off",
                                SettingsMenuButtonComponent(SettingsMenuButton::FrameLimit),
                            );
                        });
                    parent.new_menu_button(
                        "Apply",
                        SettingsMenuButtonComponent(SettingsMenuButton::Apply),
                    );
                });
        });
    }
}
fn exit_settings(settings_menu: Query<Entity, With<SettingsMenu>>, mut commands: Commands) {
    let settings_menu = settings_menu.get_single();
    if let Ok(settings_menu) = settings_menu {
        commands.entity(settings_menu).despawn_recursive();
    }
}

#[derive(Component)]
struct MainMenu;
#[derive(Component)]
struct SettingsMenu;
enum SettingsMenuButton {
    Apply,
    FrameLimit,
}

#[derive(Component)]
struct SettingsMenuButtonComponent(SettingsMenuButton);
