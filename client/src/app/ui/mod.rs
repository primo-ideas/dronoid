use bevy::{
    camera::{Camera, Camera2d, visibility::Visibility},
    input::{ButtonInput, mouse::MouseButton},
    input_focus::{
        AutoFocus,
        tab_navigation::{TabGroup, TabIndex},
    },
    math::Vec3,
    sprite::Sprite,
    text::{EditableText, EditableTextFilter, FontSize, TextCursorStyle, TextFont},
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
    palettes::css::{DARK_SLATE_GRAY, WHITE},
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

pub mod handle;
pub mod setup;

use crate::app::{ActionMessage, GameSprites, GameState, InfoMessage, PlayState, PlayerName};

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

#[derive(Component)]
pub struct FactoryInPlacement;

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

pub fn show_resources_panel(mut resources_panel: Query<&mut Visibility, With<ResourcesPanel>>) {
    let mut visibility = resources_panel.iter_mut().next().unwrap();
    *visibility.deref_mut() = Visibility::Visible;
}

pub fn show_game_panel(mut game_panel: Query<&mut Visibility, With<GamePanel>>) {
    let mut visibility = game_panel.iter_mut().next().unwrap();
    *visibility.deref_mut() = Visibility::Visible;
}
