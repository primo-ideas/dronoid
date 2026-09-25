use bevy::{
    camera::visibility::Visibility,
    ui::{BorderColor, BorderRadius, Val, widget::Text},
};
use bevy_color::Color;
use bevy_ecs::{component::Component, message::MessageReader, system::ParamSet};
use bevy_ecs::{query::With, system::Query};
use std::ops::DerefMut;

pub mod handle;
pub mod setup;

use crate::app::InfoMessage;

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
pub struct LeaveGameButton;

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

pub fn show_connect_page(
    mut visibilities: ParamSet<(
        Query<&mut Visibility, With<ConnectPage>>,
        Query<&mut Visibility, With<ResourcesPanel>>,
        Query<&mut Visibility, With<GamePanel>>,
        Query<&mut Visibility, With<LeaveGameButton>>,
    )>,
) {
    *visibilities.p0().single_mut().unwrap().deref_mut() = Visibility::Visible;
    *visibilities.p1().single_mut().unwrap().deref_mut() = Visibility::Hidden;
    *visibilities.p2().single_mut().unwrap().deref_mut() = Visibility::Hidden;
    *visibilities.p3().single_mut().unwrap().deref_mut() = Visibility::Hidden;
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

pub fn show_leave_game_button(mut button: Query<&mut Visibility, With<LeaveGameButton>>) {
    let mut visibility = button.iter_mut().next().unwrap();
    *visibility.deref_mut() = Visibility::Visible;
}
