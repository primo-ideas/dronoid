use bevy::{
    camera::{Camera, Camera2d, visibility::Visibility},
    input::{ButtonInput, mouse::MouseButton},
    input_focus::{
        AutoFocus,
        tab_navigation::{TabGroup, TabIndex},
    },
    math::Vec3,
    sprite::Sprite,
    text::{EditableText, EditableTextFilter, TextCursorStyle},
    transform::components::{GlobalTransform, Transform},
    ui::{
        AlignItems, BackgroundColor, BorderColor, BorderRadius, FlexDirection, Interaction,
        IsDefaultUiCamera, JustifyContent, Node, PositionType, UiRect, Val, percent, px,
        widget::Text,
    },
    utils::default,
    window::Window,
};
use bevy_color::{
    Color, LinearRgba,
    palettes::{
        css::{DARK_SLATE_GRAY, WHITE},
        tailwind::SLATE_300,
    },
};
use bevy_ecs::{
    children, component::Component, entity::Entity, message::MessageReader, system::Res,
};
use bevy_ecs::{
    message::MessageWriter,
    query::{Changed, With},
    system::{Commands, Query, ResMut},
};
use bevy_state::state::NextState;
use std::ops::DerefMut;

use crate::app::{ActionMessage, GameSprites, GameState, InfoMessage, PlayState, PlayerName};

#[cfg(not(target_arch = "wasm32"))]
use crate::app::ProgramOptions;

#[cfg(not(target_arch = "wasm32"))]
pub mod desktop;

#[derive(Component)]
pub struct PlayerNameField;

#[derive(Component)]
pub struct ConnectButton;

#[derive(Component)]
pub struct PlaceFactoryButton;

#[derive(Component)]
pub struct InfoLabel;

#[derive(Component)]
pub struct ConnectPage;

#[derive(Component)]
pub struct GamePanel;

#[derive(Component)]
pub struct ResourcesPanel;

// #[derive(Component)]
// pub struct UiCameraMarker;

#[derive(Component)]
pub struct FactoryInPlacement;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);

pub fn setup_ui_camera(mut commands: Commands) {
    commands.spawn((
        // UiCameraMarker,
        IsDefaultUiCamera,
        Camera2d::default(),
        Transform::from_xyz(0., 0., 0.),
    ));
}

pub fn show_connect_page(mut connect_page: Query<&mut Visibility, With<ConnectPage>>) {
    let mut connect_page_visibility = connect_page.iter_mut().next().unwrap();
    *connect_page_visibility.deref_mut() = Visibility::Visible;
}

pub fn info_label(
    mut info_events: MessageReader<InfoMessage>,
    mut info_label: Query<&mut Text, With<InfoLabel>>,
) {
    let mut info_label = info_label.iter_mut().next().unwrap();

    for info_event in info_events.read() {
        info_label.0 = info_event.0.clone();
    }
}

pub fn handle_placing_factory(
    mut factory_in_placement: Query<(Entity, &mut Transform), With<FactoryInPlacement>>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    windows: Query<&Window>,
    // play_state: Res<PlayState>,
    mut next_play_state: ResMut<NextState<PlayState>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut actions: MessageWriter<ActionMessage>,
    mut commands: Commands,
) {
    // if let PlayState::Idle = play_state.deref() {
    //     return;
    // }

    let (camera, camera_transform) = camera.single().unwrap();
    let maybe_cursor_position = windows.single().unwrap().cursor_position();
    if maybe_cursor_position.is_none() {
        return;
    }
    let cursor_position = maybe_cursor_position.unwrap();
    let (factory_entity, mut factory_sprite_transform) = factory_in_placement.single_mut().unwrap();
    let position = camera
        .viewport_to_world_2d(camera_transform, cursor_position)
        .unwrap();
    if mouse_button.just_pressed(MouseButton::Left) {
        actions.write(ActionMessage(dronoid_protocol::Action::PlaceFactory((
            position.x, position.y,
        ))));
        commands.entity(factory_entity).despawn();
        next_play_state.set(PlayState::Idle);
        return;
    }
    factory_sprite_transform.translation = Vec3::new(
        position.x,
        position.y,
        factory_sprite_transform.translation.z,
    );
}

pub fn handle_place_factory_button(
    button: Query<&Interaction, (With<PlaceFactoryButton>, Changed<Interaction>)>,
    mut play_state: ResMut<NextState<PlayState>>,
    sprites: Res<GameSprites>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut commands: Commands,
) {
    let maybe_interaction = button.iter().next();
    if maybe_interaction.is_none() {
        return;
    }
    let interaction = maybe_interaction.unwrap();
    let maybe_mouse_position = windows.iter().next().unwrap().cursor_position();
    if maybe_mouse_position.is_none() {
        return;
    }
    let mouse_position = maybe_mouse_position.unwrap();
    let (camera, camera_position) = camera.iter().next().unwrap();
    let position = camera
        .viewport_to_world_2d(camera_position, mouse_position)
        .unwrap();
    match *interaction {
        Interaction::Pressed => {
            let (size, image_hdl) = sprites.0.get(&dronoid_protocol::Kind::Factory).unwrap();
            let mut transform = Transform::from_xyz(position.x, position.y, 100.);
            transform.scale.x = *size;
            transform.scale.y = *size;
            commands.spawn((
                FactoryInPlacement,
                transform,
                Sprite::from_image(image_hdl.clone()),
            ));
            play_state.set(PlayState::PlacingFactory);
        }
        Interaction::Hovered => {
            // *color = HOVERED_BUTTON.into();
        }
        Interaction::None => {
            // input_focus.clear();
            // *color = NORMAL_BUTTON.into();
        }
    }
}

pub fn setup_resources_panel(mut commands: Commands) {
    commands.spawn((
        ResourcesPanel,
        Visibility::Hidden,
        BackgroundColor {
            0: Color::LinearRgba(LinearRgba::rgb(0.1, 0.1, 0.1)),
        },
        Node {
            width: percent(30.),
            height: percent(20.),
            padding: percent(2.).all(),
            margin: percent(2.).all(),
            left: px(0),
            top: px(0),
            position_type: PositionType::Absolute,
            border: px(2.).all(),
            border_radius: BorderRadius::all(Val::Percent(10.)),
            ..default()
        },
        children![(Text::new("Minerals   "),), (Text::new("<nb_minerals>"),)],
    ));
}

pub fn show_resources_panel(mut resources_panel: Query<&mut Visibility, With<ResourcesPanel>>) {
    let mut visibility = resources_panel.iter_mut().next().unwrap();
    *visibility.deref_mut() = Visibility::Visible;
}

pub fn setup_game_panel(mut commands: Commands) {
    commands
        .spawn((
            GamePanel,
            Visibility::Hidden,
            BackgroundColor {
                0: Color::LinearRgba(LinearRgba::rgb(0.1, 0.1, 0.1)),
            },
            Node {
                width: percent(20.),
                height: percent(60.),
                padding: percent(2.).all(),
                margin: percent(2.).all(),
                right: px(0),
                top: px(0),
                position_type: PositionType::Absolute,
                border: px(2.).all(),
                border_radius: BorderRadius::all(Val::Percent(10.)),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                PlaceFactoryButton,
                Interaction::default(),
                Node {
                    flex_grow: 1.,
                    height: px(30),
                    border: UiRect::all(px(2)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border_radius: BorderRadius::all(Val::Percent(10.)),
                    ..default()
                },
                BorderColor::all(WHITE),
                children![(Text::new("Spawn factory"),)],
            ));
        });
}

pub fn show_game_panel(mut game_panel: Query<&mut Visibility, With<GamePanel>>) {
    let mut visibility = game_panel.iter_mut().next().unwrap();
    *visibility.deref_mut() = Visibility::Visible;
}

pub fn handle_connect_button(
    connect_button: Query<
        // Entity,
        // &ConnectButton,
        &Interaction,
        (With<ConnectButton>, Changed<Interaction>),
    >,
    player_name_field: Query<&EditableText, With<PlayerNameField>>,
    mut info_label: MessageWriter<InfoMessage>,
    // mut input_focus: ResMut<InputFocus>,
    mut player_name: ResMut<PlayerName>,
    mut state: ResMut<NextState<GameState>>,
) {
    let maybe_interaction = connect_button.iter().next();
    if maybe_interaction.is_none() {
        return;
    }
    let interaction = maybe_interaction.unwrap();
    let player_name_text = player_name_field.iter().next().unwrap();
    // for (entity, _, interaction, mut color, _, _) in &mut connect_button {
    // for interaction in &mut connect_button {
    match *interaction {
        Interaction::Pressed => {
            let player_name_field_string = player_name_text.value().to_string();
            player_name.0 = player_name_field_string;
            info_label.write(InfoMessage("Connecting...".to_string()));
            state.set(GameState::Connect);
            // input_focus.set(entity, FocusCause::Pressed);
        }
        Interaction::Hovered => {
            // *color = HOVERED_BUTTON.into();
        }
        Interaction::None => {
            // input_focus.clear();
            // *color = NORMAL_BUTTON.into();
        }
    }
    // }
}

pub fn handle_buttons(
    mut connect_button: Query<(&Interaction, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (interaction, mut background_color) in &mut connect_button {
        match *interaction {
            Interaction::Pressed => {
                // let player_name_field_string = player_name_text.value().to_string();
                // player_name.0 = player_name_field_string;
                // info_label.write(InfoMessage("Connecting...".to_string()));
                // state.set(State::Connect);
                // input_focus.set(entity, FocusCause::Pressed);
            }
            Interaction::Hovered => {
                *background_color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                // input_focus.clear();
                *background_color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub fn setup_connect_page(
    #[cfg(not(target_arch = "wasm32"))] program_options: Res<ProgramOptions>,
    player_name: Res<PlayerName>,
    mut state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    let mut player_name_editable_text = EditableText::new(player_name.0.to_string().as_str());
    player_name_editable_text.cursor_width = 0.4;
    player_name_editable_text.max_characters = Some(20);
    commands
        .spawn((
            Node {
                width: percent(100.),
                height: percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Visibility::Visible,
            // ConnectPage,
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: px(20.),
                    ..default()
                },
                InfoLabel,
                Text::new(""),
            ));
        });

    commands
        .spawn((
            Node {
                width: percent(100.),
                height: percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Visibility::Visible,
            ConnectPage,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(10.)),
                        row_gap: Val::Px(10.),
                        flex_direction: FlexDirection::Column,
                        border: px(2.).all(),
                        border_radius: BorderRadius::all(Val::Percent(10.)),
                        ..default()
                    },
                    BorderColor::all(bevy::color::palettes::css::WHITE),
                    TabGroup::new(0),
                ))
                .with_children(|parent| {
                    #[cfg(not(target_arch = "wasm32"))]
                    desktop::setup_host_port(parent, program_options);

                    parent
                        .spawn(Node {
                            column_gap: Val::Px(10.),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                Node {
                                    ..Default::default()
                                },
                                Text::new("Player name"),
                            ));
                            parent.spawn((
                                PlayerNameField,
                                Node {
                                    width: px(200),
                                    border: px(2.).all(),
                                    border_radius: BorderRadius::all(Val::Percent(10.)),
                                    ..default()
                                },
                                player_name_editable_text,
                                AutoFocus,
                                TabIndex(2),
                                TextCursorStyle {
                                    color: bevy_color::Color::Srgba(WHITE),
                                    ..Default::default()
                                },
                                EditableTextFilter::new(|c| c.is_ascii_alphabetic()),
                                BackgroundColor(DARK_SLATE_GRAY.into()),
                                BorderColor::all(SLATE_300),
                            ));
                            parent.spawn((
                                ConnectButton,
                                Interaction::default(),
                                TabIndex(3),
                                Node {
                                    flex_grow: 1.,
                                    border: UiRect::all(px(2)),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    border_radius: BorderRadius::all(Val::Percent(10.)),
                                    ..default()
                                },
                                BorderColor::all(WHITE),
                                children![(Text::new("Connect"),)],
                            ));
                        });
                });
        });

    state.set(GameState::HandleConnectPage);
}
