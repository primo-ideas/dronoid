use bevy::{
    camera::{Camera, Camera2d},
    input::{ButtonInput, mouse::MouseButton},
    math::Vec3,
    sprite::Sprite,
    text::EditableText,
    transform::components::{GlobalTransform, Transform},
    ui::{BackgroundColor, Interaction},
    window::Window,
};
use bevy_ecs::{entity::Entity, system::Res};
use bevy_ecs::{
    message::MessageWriter,
    query::{Changed, With},
    system::{Commands, Query, ResMut},
};
use bevy_state::state::NextState;

use crate::app::{
    ActionMessage, GameSprites, GameState, InfoMessage, PlayState, PlayerName,
    ui::{
        ConnectButton, FactoryInPlacement, HOVERED_BUTTON, NORMAL_BUTTON, PlaceFactoryButton,
        PlayerNameField,
    },
};

pub fn placing_factory(
    mut factory_in_placement: Query<(Entity, &mut Transform), With<FactoryInPlacement>>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    windows: Query<&Window>,
    mut next_play_state: ResMut<NextState<PlayState>>,
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
        next_play_state.set(PlayState::Idle);
        return;
    }
    factory_sprite_transform.translation = Vec3::new(
        position.x,
        position.y,
        factory_sprite_transform.translation.z,
    );
}

pub fn place_factory_button(
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
        Interaction::Hovered => {}
        Interaction::None => {}
    }
}

pub fn connect_button(
    connect_button: Query<&Interaction, (With<ConnectButton>, Changed<Interaction>)>,
    player_name_field: Query<&EditableText, With<PlayerNameField>>,
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
