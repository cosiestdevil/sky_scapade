
use crate::settings::*;
use crate::{discord::ActivityState, UiHelper};
use bevy::{
    app::AppExit,
    prelude::*
};
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(MainMenuState::Menu);
        app.add_systems(
            Update,
            (main_menu_button_system, settings_menu_button_system),
        );
        app.add_systems(OnEnter(crate::AppState::MainMenu), enter_main_menu);
        app.add_systems(OnExit(crate::AppState::MainMenu), exit_main_menu);
        app.add_systems(OnEnter(MainMenuState::Settings), enter_settings);
        app.add_systems(OnExit(MainMenuState::Settings), exit_settings);
    }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MainMenuState {
    Menu,
    #[allow(dead_code)]
    NewGame,
    Settings,
}
enum MainMenuButton {
    NewGame,
    Settings,
    Exit,
}
#[derive(Component)]
struct MainMenuButtonComponent(MainMenuButton);

type SettingsMenuButtonType<'a> = (
    &'a Interaction,
    &'a mut BackgroundColor,
    &'a mut BorderColor,
    &'a Children,
    &'a SettingsMenuButtonComponent,
);
fn settings_menu_button_system(
    mut interaction_query: Query<SettingsMenuButtonType, ButtonInteractionFilter>,
    mut settings: ResMut<crate::settings::SettingsResource>,
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
                        let new_limit = settings.frame_limit.next();
                        settings
                            .update(|settings| settings.frame_limit = new_limit)
                            .unwrap();
                        text.sections[0].value = new_limit.label().into();
                    }
                    SettingsMenuButton::WindowMode => {
                        let new_mode = settings.window_mode.next();
                        settings
                            .update(|settings| {
                                settings.window_mode = new_mode;
                            })
                            .unwrap();
                        text.sections[0].value = new_mode.label().into();
                    }
                    SettingsMenuButton::AntiAlias => {
                        let new_mode = settings.anti_alias.next();
                        settings
                            .update(|settings| {
                                settings.anti_alias = new_mode;
                            })
                            .unwrap();
                        text.sections[0].value = new_mode.label().into();
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

type ButtonInteractionFilter = (Changed<Interaction>, With<Button>);
fn main_menu_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
            &MainMenuButtonComponent,
        ),
        ButtonInteractionFilter,
    >,
    mut next_state: ResMut<NextState<crate::AppState>>,
    mut next_menu: ResMut<NextState<MainMenuState>>,
    mut text_query: Query<&mut Text>,
    mut exit: EventWriter<AppExit>,
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
                    MainMenuButton::Exit => {
                        exit.send(AppExit);
                    }
                };
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
fn enter_main_menu(
    safe_ui: Query<Entity, With<crate::SafeUi>>,
    mut commands: Commands,
    mut discord_activity: ResMut<ActivityState>,
) {
    discord_activity.state = Some("In Main Menu".into());
    discord_activity.details = None;
    discord_activity.start = None;

    let safe_ui = safe_ui.get_single();
    if let Ok(safe_ui) = safe_ui {
        let mut safe_ui = commands.entity(safe_ui);
        safe_ui.despawn_descendants();
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
                            parent.new_menu_button(
                                "Exit",
                                MainMenuButtonComponent(MainMenuButton::Exit),
                            );
                        });
                });
        });
    }
}
type ExitMainMenuFilter = With<MainMenu>;
fn exit_main_menu(main_menu: Query<Entity, ExitMainMenuFilter>, mut commands: Commands) {
    let main_menu = main_menu.iter();
    for main_menu in main_menu {
        commands.entity(main_menu).despawn_recursive();
    }
}

fn enter_settings(
    main_menu: Query<Entity, With<MainMenu>>,
    mut commands: Commands,
    settings: Res<crate::settings::SettingsResource>,
) {
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
                        .with_children(|window_mode| {
                            window_mode.spawn(TextBundle::from_section(
                                "Window Mode: ",
                                TextStyle {
                                    color: Color::WHITE,
                                    font_size: 42.0,
                                    ..default()
                                },
                            ));
                            window_mode.new_menu_button(
                                settings.window_mode.label(),
                                SettingsMenuButtonComponent(SettingsMenuButton::WindowMode),
                            );
                        });
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
                                settings.frame_limit.label(),
                                SettingsMenuButtonComponent(SettingsMenuButton::FrameLimit),
                            );
                        });
                    parent
                        .spawn(NodeBundle::default())
                        .with_children(|frame_rate| {
                            frame_rate.spawn(TextBundle::from_section(
                                "Anti Aliasing: ",
                                TextStyle {
                                    color: Color::WHITE,
                                    font_size: 42.0,
                                    ..default()
                                },
                            ));
                            frame_rate.new_menu_button(
                                settings.anti_alias.label(),
                                SettingsMenuButtonComponent(SettingsMenuButton::AntiAlias),
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
    WindowMode,
    AntiAlias,
}

#[derive(Component)]
struct SettingsMenuButtonComponent(SettingsMenuButton);
