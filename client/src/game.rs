use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
    text::TextSection,
};
use std::collections::HashMap;

use crate::{GameState, LeaveGameMessage, net::StateMessage, ui::ResourcesTextMarker};

#[derive(Resource, Default)]
pub struct SpawnPoint(pub (f32, f32));

#[derive(Resource, Default)]
pub struct Entities(pub HashMap<u32, Entity>);

#[derive(Resource, Default)]
pub struct GameSprites(pub HashMap<dronoid_protocol::Kind, (f32, Handle<Image>)>);

pub fn plugin(app: &mut App) {
    app.init_resource::<SpawnPoint>();
    app.init_resource::<Entities>();
    app.init_resource::<GameSprites>();
    app.add_message::<LeaveGameMessage>();
    app.add_systems(Update, prepare.run_if(in_state(GameState::PrepareGame)));
    app.add_systems(Update, handle_camera.run_if(in_state(GameState::Play)));
    app.add_systems(Update, show_game.run_if(in_state(GameState::Play)));
    app.add_systems(Update, leave_game.run_if(in_state(GameState::Play)));
}

fn leave_game(
    mut leave_messages: MessageReader<LeaveGameMessage>,
    mut next_state: ResMut<NextState<GameState>>,
    mut entities: ResMut<Entities>,
) {
    if leave_messages.read().count() > 0 {
        leave_messages.clear();
        entities.0.clear();
        next_state.set(GameState::Welcome);
    }
}

fn prepare(
    mut camera: Query<(&mut Projection, &mut Transform), With<Camera2d>>,
    mut state: ResMut<NextState<GameState>>,
    spawn_point: Res<SpawnPoint>,
) {
    let (mut projection, mut transform) = camera.iter_mut().next().unwrap();
    if let Projection::Orthographic(ref mut ortho) = *projection {
        ortho.scale = 0.1;
    }

    *transform = Transform::from_xyz(spawn_point.0.0, spawn_point.0.1, 0.);
    state.set(GameState::Play);
}

fn handle_camera(
    mut camera: Query<(&mut Projection, &mut Transform), With<Camera2d>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard_button: Res<ButtonInput<KeyCode>>,
    spawn_point: Res<SpawnPoint>,
) {
    let (mut projection, mut transform) = camera.iter_mut().next().unwrap();

    if let Projection::Orthographic(ortho) = projection.as_mut() {
        if mouse_button.pressed(MouseButton::Right) {
            let factor = 10.;
            transform.translation.x += mouse_motion.delta.x / factor;
            transform.translation.y -= mouse_motion.delta.y / factor;
        } else {
            let zoom_speed = 0.2;
            match mouse_scroll.unit {
                bevy::input::mouse::MouseScrollUnit::Pixel => {
                    let delta = mouse_scroll.delta.y;

                    ortho.scale *= 1.0 - (delta * zoom_speed);
                }
                bevy::input::mouse::MouseScrollUnit::Line => {
                    ortho.scale *= 1.0 - (mouse_scroll.delta.y * zoom_speed * 0.5);
                }
            }
            ortho.scale = ortho.scale.clamp(0.05, 80.0);
        }

        if keyboard_button.just_pressed(KeyCode::Space) {
            transform.translation.x = spawn_point.0.0;
            transform.translation.y = spawn_point.0.1;
        }
    }
}

fn show_game(
    mut state_messages: MessageReader<StateMessage>,
    mut entities: Query<&mut Transform>,
    mut resources_text: Query<&mut Text, With<ResourcesTextMarker>>,
    mut r_entities: ResMut<Entities>,
    r_sprites: Res<GameSprites>,
    mut commands: Commands,
) {
    for state_message in state_messages.read() {
        *resources_text.single_mut().unwrap().get_text_mut() =
            state_message.0.minerals_cnt.to_string();
        for entity_state in state_message.0.entities_in_zone.iter() {
            if let Some(existing_entity) = r_entities.0.get(&entity_state.id) {
                if let Ok(mut transform) = entities.get_mut(*existing_entity) {
                    transform.translation.x = entity_state.pos.0;
                    transform.translation.y = entity_state.pos.1;
                }
            } else {
                let (size, image_hdl) = r_sprites.0.get(&entity_state.kind).unwrap();
                let mut transform = Transform::from_xyz(entity_state.pos.0, entity_state.pos.1, 0.);
                transform.scale.x = *size;
                transform.scale.y = *size;
                let entity = commands.spawn((
                    DespawnOnExit(GameState::Play),
                    transform,
                    Sprite::from_image(image_hdl.clone()),
                ));
                r_entities.0.insert(entity_state.id, entity.id());
            }
        }
    }
}
