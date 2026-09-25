use bevy::{
    ecs::relationship::RelatedSpawnerCommands,
    input_focus::{
        AutoFocus,
        tab_navigation::{TabGroup, TabIndex},
    },
    prelude::*,
    text::{EditableText, EditableTextFilter, TextCursorStyle},
};
use bevy_color::palettes::css::{DARK_SLATE_GRAY, WHITE};
use rand::random_range;

use crate::game::{GameSprites, GameState, ProgramOptions};

#[derive(Component)]
pub struct HostField;

#[derive(Component)]
pub struct PortField;

#[derive(Message)]
pub struct InfoMessage(pub String);

#[derive(Resource, States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum UiState {
    #[default]
    Idle,
    PlacingFactory,
}

#[derive(Resource)]
pub struct PlayerName(pub String);

const FONT_SIZE: f32 = 4.;
const PADDING: f32 = 3.;
const BORDER_RADIUS: f32 = 8.;
const BORDER_THICKNESS: f32 = 2.2;
const BORDER_COLOR: Color = Color::srgb_u8(0x34, 0xc6, 0xeb);
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);

pub fn border_radius() -> BorderRadius {
    BorderRadius::all(Val::Px(BORDER_RADIUS))
}

pub fn border_color() -> BorderColor {
    BorderColor::all(BORDER_COLOR)
}

#[derive(Component)]
pub struct PlayerNameFieldMarker;

#[derive(Component)]
pub struct LeaveGameButtonMarker;

#[derive(Component)]
pub struct ConnectButtonMarker;

#[derive(Component)]
pub struct PlaceFactoryButtonMarker;

#[derive(Component)]
pub struct InfoLabelMarker;

#[derive(Component)]
pub struct ConnectPageMarker;

#[derive(Component)]
pub struct GamePanelMarker;

#[derive(Component)]
pub struct ResourcesPanelMarker;

#[derive(Component)]
pub struct FactoryInPlacementMarker;

fn gen_name() -> String {
    format!("Player{}", random_range(u8::MIN..u8::MAX)).to_string()
}

pub fn setup_sprites(asset_server: Res<AssetServer>, mut game_sprites: ResMut<GameSprites>) {
    game_sprites.0.insert(
        dronoid_protocol::Kind::Mineral,
        (1. / 128., asset_server.load("textures/mineral.png")),
    );
    game_sprites.0.insert(
        dronoid_protocol::Kind::Dronoid,
        (3. / 128., asset_server.load("textures/dronoid.png")),
    );
    game_sprites.0.insert(
        dronoid_protocol::Kind::Factory,
        (6. / 128., asset_server.load("textures/factory.png")),
    );
    game_sprites.0.insert(
        dronoid_protocol::Kind::Spawn,
        (9. / 128., asset_server.load("textures/spawn.png")),
    );
}

pub fn setup_host_port(
    parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
    program_options: Res<ProgramOptions>,
) {
    let mut host_editable_text = EditableText::new(program_options.hostname.to_string().as_str());
    host_editable_text.cursor_width = 0.4;
    host_editable_text.max_characters = Some(62);
    let mut port_editable_text = EditableText::new(program_options.port.to_string().as_str());
    port_editable_text.cursor_width = 0.4;
    port_editable_text.max_characters = Some(5);

    parent
        .spawn(Node {
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Node {
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                Text::new("Host"),
                TextFont::from_font_size(FontSize::VMin(FONT_SIZE)),
            ));
            parent.spawn((
                HostField,
                Node {
                    padding: px(PADDING).all(),
                    width: px(200),
                    border: px(BORDER_THICKNESS).all(),
                    border_radius: border_radius(),
                    ..default()
                },
                host_editable_text,
                TextFont::from_font_size(FontSize::VMin(FONT_SIZE)),
                TabIndex(0),
                TextCursorStyle {
                    color: bevy_color::Color::Srgba(WHITE),
                    ..Default::default()
                },
                EditableTextFilter::new(|c| c.is_ascii() && c.is_ascii_graphic()),
                BackgroundColor(DARK_SLATE_GRAY.into()),
                border_color(),
                AutoFocus,
            ));
            parent.spawn((
                Node {
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                Text::new("Port"),
                TextFont::from_font_size(FontSize::VMin(FONT_SIZE)),
            ));
            parent.spawn((
                PortField,
                Node {
                    padding: px(PADDING).all(),
                    width: px(80),
                    border: px(BORDER_THICKNESS).all(),
                    border_radius: border_radius(),
                    ..default()
                },
                port_editable_text,
                TextFont::from_font_size(FontSize::VMin(FONT_SIZE)),
                TabIndex(1),
                TextCursorStyle {
                    color: bevy_color::Color::Srgba(WHITE),
                    ..Default::default()
                },
                EditableTextFilter::new(|c| c.is_ascii() && c.is_ascii_graphic() && c.is_numeric()),
                BackgroundColor(DARK_SLATE_GRAY.into()),
                border_color(),
            ));
        });
}

pub fn ui_camera(mut commands: Commands) {
    commands.spawn((
        IsDefaultUiCamera,
        Camera2d::default(),
        Transform::from_xyz(0., 0., 0.),
    ));
}

pub fn resources_panel(mut commands: Commands) {
    commands
        .spawn((
            ResourcesPanelMarker,
            Visibility::Hidden,
            BackgroundColor {
                0: Color::LinearRgba(LinearRgba::rgb(0.1, 0.1, 0.1)),
            },
            Node {
                width: percent(30.),
                height: percent(20.),
                padding: percent(PADDING).all(),
                margin: percent(2.).all(),
                left: px(0),
                top: px(0),
                position_type: PositionType::Absolute,
                border: px(BORDER_THICKNESS).all(),
                border_radius: border_radius(),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Minerals:"),
                TextFont::from_font_size(FontSize::VMin(FONT_SIZE)),
            ));
            parent.spawn((
                Text::new("<nb>"),
                TextFont::from_font_size(FontSize::VMin(FONT_SIZE)),
            ));
        });
}

pub fn setup_leave_game_button(mut commands: Commands) {
    commands.spawn((
        LeaveGameButtonMarker,
        Visibility::Hidden,
        BackgroundColor {
            0: Color::LinearRgba(LinearRgba::rgb(0.1, 0.1, 0.1)),
        },
        Node {
            // width: percent(20.),
            // height: percent(60.),
            // padding: percent(PADDING).all(),
            // margin: percent(2.).all(),
            left: px(0),
            bottom: px(0),
            position_type: PositionType::Absolute,
            border: px(BORDER_THICKNESS).all(),
            border_radius: border_radius(),
            ..default()
        },
        Interaction::default(),
        border_color(),
        children![(
            Text::new("Leave"),
            TextFont::from_font_size(FontSize::VMin(FONT_SIZE)),
        )],
    ));
}

pub fn game_panel(mut commands: Commands) {
    commands
        .spawn((
            GamePanelMarker,
            Visibility::Hidden,
            BackgroundColor {
                0: Color::LinearRgba(LinearRgba::rgb(0.1, 0.1, 0.1)),
            },
            Node {
                width: percent(20.),
                height: percent(60.),
                padding: percent(PADDING).all(),
                margin: percent(2.).all(),
                right: px(0),
                top: px(0),
                position_type: PositionType::Absolute,
                border: px(BORDER_THICKNESS).all(),
                border_radius: border_radius(),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                PlaceFactoryButtonMarker,
                Interaction::default(),
                Node {
                    flex_grow: 1.,
                    height: px(30),
                    border: UiRect::all(px(BORDER_THICKNESS)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border_radius: border_radius(),
                    ..default()
                },
                border_color(),
                children![(
                    Text::new("Spawn factory"),
                    TextFont::from_font_size(FontSize::VMin(FONT_SIZE)),
                )],
            ));
        });
}

pub fn connect_page(
    program_options: Res<ProgramOptions>,
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
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: px(20.),
                    ..default()
                },
                InfoLabelMarker,
                Text::new(""),
                TextFont::from_font_size(FontSize::VMin(FONT_SIZE)),
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
            ConnectPageMarker,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(PADDING)),
                        flex_direction: FlexDirection::Column,
                        border: px(BORDER_THICKNESS).all(),
                        border_radius: border_radius(),
                        ..default()
                    },
                    border_color(),
                    TabGroup::new(0),
                ))
                .with_children(|parent| {
                    setup_host_port(parent, program_options);

                    parent.spawn(Node { ..default() }).with_children(|parent| {
                        parent.spawn((
                            Node {
                                padding: px(PADDING).all(),
                                align_items: AlignItems::Center,
                                ..Default::default()
                            },
                            Text::new("Player name"),
                            TextFont::from_font_size(FontSize::VMin(FONT_SIZE)),
                        ));
                        parent.spawn((
                            PlayerNameFieldMarker,
                            Node {
                                padding: px(PADDING).all(),
                                width: px(200),
                                border: px(BORDER_THICKNESS).all(),
                                border_radius: border_radius(),
                                ..default()
                            },
                            player_name_editable_text,
                            TextFont::from_font_size(FontSize::VMin(FONT_SIZE)),
                            AutoFocus,
                            TabIndex(2),
                            TextCursorStyle {
                                color: bevy_color::Color::Srgba(WHITE),
                                ..Default::default()
                            },
                            EditableTextFilter::new(|c| c.is_ascii_alphabetic()),
                            BackgroundColor(DARK_SLATE_GRAY.into()),
                            border_color(),
                        ));
                        parent.spawn((
                            ConnectButtonMarker,
                            Interaction::default(),
                            TabIndex(3),
                            Node {
                                padding: px(PADDING).all(),
                                flex_grow: 1.,
                                border: UiRect::all(px(BORDER_THICKNESS)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border_radius: border_radius(),
                                ..default()
                            },
                            border_color(),
                            children![(
                                Text::new("Connect"),
                                TextFont::from_font_size(FontSize::VMin(FONT_SIZE)),
                            )],
                        ));
                    });
                });
        });

    state.set(GameState::HandleConnectPage);
}

pub fn placing_factory(
    mut factory_in_placement: Query<(Entity, &mut Transform), With<FactoryInPlacementMarker>>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    windows: Query<&Window>,
    mut next_play_state: ResMut<NextState<UiState>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut actions: MessageWriter<ActionMessage>,
    mut commands: Commands,
) {
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
        next_play_state.set(UiState::Idle);
        return;
    }
    factory_sprite_transform.translation = Vec3::new(
        position.x,
        position.y,
        factory_sprite_transform.translation.z,
    );
}

pub fn place_factory_button(
    button: Query<&Interaction, (With<PlaceFactoryButtonMarker>, Changed<Interaction>)>,
    mut play_state: ResMut<NextState<UiState>>,
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
                FactoryInPlacementMarker,
                transform,
                Sprite::from_image(image_hdl.clone()),
            ));
            play_state.set(UiState::PlacingFactory);
        }
        Interaction::Hovered => {}
        Interaction::None => {}
    }
}

pub fn connect_button(
    connect_button: Query<&Interaction, (With<ConnectButtonMarker>, Changed<Interaction>)>,
    player_name_field: Query<&EditableText, With<PlayerNameFieldMarker>>,
    mut info_label: MessageWriter<InfoMessage>,
    mut player_name: ResMut<PlayerName>,
    mut state: ResMut<NextState<GameState>>,
) {
    let maybe_interaction = connect_button.iter().next();
    if maybe_interaction.is_none() {
        return;
    }
    let interaction = maybe_interaction.unwrap();
    let player_name_text = player_name_field.iter().next().unwrap();
    match *interaction {
        Interaction::Pressed => {
            let player_name_field_string = player_name_text.value().to_string();
            player_name.0 = player_name_field_string;
            info_label.write(InfoMessage("Connecting...".to_string()));
            state.set(GameState::Connect);
        }
        _ => {}
    }
}

pub fn leave_game_button(
    button: Query<&Interaction, (With<LeaveGameButtonMarker>, Changed<Interaction>)>,
    // player_name_field: Query<&EditableText, With<PlayerNameField>>,
    // mut info_label: MessageWriter<InfoMessage>,
    // mut player_name: ResMut<PlayerName>,
    mut state: ResMut<NextState<GameState>>,
) {
    let maybe_interaction = button.iter().next();
    if maybe_interaction.is_none() {
        return;
    }
    let interaction = maybe_interaction.unwrap();
    // let player_name_text = player_name_field.iter().next().unwrap();
    match *interaction {
        Interaction::Pressed => {
            // let player_name_field_string = player_name_text.value().to_string();
            // player_name.0 = player_name_field_string;
            // info_label.write(InfoMessage("Connecting...".to_string()));
            state.set(GameState::ShowConnectPage);
        }
        _ => {}
    }
}

pub fn buttons(
    mut connect_button: Query<(&Interaction, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (interaction, mut background_color) in &mut connect_button {
        match *interaction {
            Interaction::Hovered => {
                *background_color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *background_color = NORMAL_BUTTON.into();
            }
            _ => {}
        }
    }
}

pub fn show_connect_page(
    mut visibilities: ParamSet<(
        Query<&mut Visibility, With<ConnectPageMarker>>,
        Query<&mut Visibility, With<ResourcesPanelMarker>>,
        Query<&mut Visibility, With<GamePanelMarker>>,
        Query<&mut Visibility, With<LeaveGameButtonMarker>>,
    )>,
) {
    *visibilities.p0().single_mut().unwrap().deref_mut() = Visibility::Visible;
    *visibilities.p1().single_mut().unwrap().deref_mut() = Visibility::Hidden;
    *visibilities.p2().single_mut().unwrap().deref_mut() = Visibility::Hidden;
    *visibilities.p3().single_mut().unwrap().deref_mut() = Visibility::Hidden;
}

pub fn info_label(
    mut info_events: MessageReader<InfoMessage>,
    mut info_label: Query<&mut Text, With<InfoLabelMarker>>,
) {
    let mut info_label = info_label.iter_mut().next().unwrap();

    for info_event in info_events.read() {
        info_label.0 = info_event.0.clone();
    }
}

pub fn show_resources_panel(
    mut resources_panel: Query<&mut Visibility, With<ResourcesPanelMarker>>,
) {
    let mut visibility = resources_panel.iter_mut().next().unwrap();
    *visibility.deref_mut() = Visibility::Visible;
}

pub fn show_game_panel(mut game_panel: Query<&mut Visibility, With<GamePanelMarker>>) {
    let mut visibility = game_panel.iter_mut().next().unwrap();
    *visibility.deref_mut() = Visibility::Visible;
}

pub fn show_leave_game_button(mut button: Query<&mut Visibility, With<LeaveGameButtonMarker>>) {
    let mut visibility = button.iter_mut().next().unwrap();
    *visibility.deref_mut() = Visibility::Visible;
}
