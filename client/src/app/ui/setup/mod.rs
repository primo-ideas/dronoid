use bevy::{
    camera::{Camera2d, visibility::Visibility},
    input_focus::{
        AutoFocus,
        tab_navigation::{TabGroup, TabIndex},
    },
    text::{EditableText, EditableTextFilter, FontSize, TextCursorStyle, TextFont},
    transform::components::Transform,
    ui::{
        AlignItems, BackgroundColor, FlexDirection, Interaction, IsDefaultUiCamera, JustifyContent,
        Node, PositionType, UiRect, Val, percent, px, widget::Text,
    },
    utils::default,
};
use bevy_color::{
    Color, LinearRgba,
    palettes::css::{DARK_SLATE_GRAY, WHITE},
};
use bevy_ecs::system::{Commands, ResMut};
use bevy_ecs::{children, system::Res};
use bevy_state::state::NextState;

#[cfg(not(target_arch = "wasm32"))]
use crate::app::ProgramOptions;
use crate::app::{
    GameState, PlayerName,
    ui::{
        BORDER_THICKNESS, ConnectButton, ConnectPage, FONT_SIZE, GamePanel, InfoLabel,
        LeaveGameButton, PADDING, PlaceFactoryButton, PlayerNameField, ResourcesPanel,
        border_color, border_radius,
    },
};

#[cfg(not(target_arch = "wasm32"))]
pub mod desktop;

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
            ResourcesPanel,
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

pub fn leave_game_button(mut commands: Commands) {
    commands.spawn((
        LeaveGameButton,
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
            GamePanel,
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
                PlaceFactoryButton,
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
            ConnectPage,
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
                    #[cfg(not(target_arch = "wasm32"))]
                    desktop::host_port(parent, program_options);

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
                            PlayerNameField,
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
                            ConnectButton,
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
