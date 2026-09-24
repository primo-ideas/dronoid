use bevy::prelude::*;
use bevy::{
    asset::AssetMetaCheck, input_focus::tab_navigation::TabNavigationPlugin, log::LogPlugin,
};
use bevy_app::{App, PluginGroup, Update};
use rand::random_range;
use std::collections::HashMap;

mod net;
mod play;
mod ui;

#[cfg(not(target_arch = "wasm32"))]
#[path = "desktop.rs"]
pub mod platform;
#[cfg(target_arch = "wasm32")]
#[path = "web.rs"]
pub mod platform;

#[derive(Message)]
pub struct ServerMessage(pub dronoid_protocol::ServerMessage);

#[derive(Message)]
pub struct ActionMessage(pub dronoid_protocol::Action);

#[derive(Message)]
pub struct InfoMessage(pub String);

#[derive(Resource, Default)]
pub struct SpawnPoint(pub (f32, f32));

#[derive(Resource, Default)]
pub struct Entities(pub HashMap<u32, Entity>);

#[derive(Resource, Default)]
pub struct GameSprites(pub HashMap<dronoid_protocol::Kind, (f32, Handle<Image>)>);

#[derive(Resource)]
pub struct ProgramOptions {
    pub hostname: String,
    pub port: u16,
    pub tls: bool,
    pub suffix: String,
}

#[derive(Resource, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    ShowConnectPage,
    HandleConnectPage,
    Connect,
    AuthenticateSendRequest,
    AuthenticateWaitResponse,
    PrepareGame,
    ShowGame,
}

impl Default for GameState {
    fn default() -> Self {
        Self::ShowConnectPage
    }
}

#[derive(Resource, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlayState {
    Idle,
    PlacingFactory,
}

impl Default for PlayState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Resource)]
pub struct PlayerName(pub String);

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

fn gen_name() -> String {
    format!("Player{}", random_range(u8::MIN..u8::MAX)).to_string()
}

pub fn run() {
    let (player_name, hostname, port, secure, suffix) = platform::init();
    App::new()
        .add_plugins((
            DefaultPlugins
                .build()
                .disable::<LogPlugin>()
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: bevy::window::PresentMode::Immediate,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            TabNavigationPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0., 0., 0.)))
        .insert_resource(Entities::default())
        .insert_resource(GameSprites::default())
        .insert_resource(PlayerName(player_name))
        .insert_resource(SpawnPoint::default())
        .insert_resource(ProgramOptions {
            suffix: suffix,
            tls: secure,
            port: port,
            hostname,
        })
        .init_state::<GameState>()
        .init_state::<PlayState>()
        .add_message::<ServerMessage>()
        .add_message::<ActionMessage>()
        .add_message::<InfoMessage>()
        .add_systems(PreStartup, platform::setup_display)
        .add_systems(Startup, setup_sprites)
        .add_systems(Startup, ui::setup_ui_camera)
        .add_systems(Startup, ui::setup_connect_page)
        .add_systems(Startup, ui::setup_game_panel)
        .add_systems(Startup, ui::setup_resources_panel)
        .add_systems(Update, ui::info_label)
        .add_systems(Update, ui::handle_buttons)
        .add_systems(
            Update,
            ui::handle_place_factory_button
                .run_if(in_state(GameState::ShowGame).and_eager(in_state(PlayState::Idle))),
        )
        .add_systems(
            Update,
            ui::handle_placing_factory.run_if(
                in_state(GameState::ShowGame).and_eager(in_state(PlayState::PlacingFactory)),
            ),
        )
        .add_systems(
            Update,
            ui::show_connect_page.run_if(in_state(GameState::ShowConnectPage)),
        )
        .add_systems(
            Update,
            ui::handle_connect_button.run_if(in_state(GameState::HandleConnectPage)),
        )
        .add_systems(
            Update,
            net::platform::connect.run_if(in_state(GameState::Connect)),
        )
        .add_systems(
            Update,
            net::platform::authenticate_send_request.run_if(
                in_state(GameState::AuthenticateSendRequest)
                    .and_then(resource_exists::<net::platform::Connection>),
            ),
        )
        .add_systems(
            Update,
            net::platform::authenticate_wait_response.run_if(
                in_state(GameState::AuthenticateWaitResponse)
                    .and_then(resource_exists::<net::platform::Connection>),
            ),
        )
        .add_systems(
            Update,
            net::platform::read_server_messages.run_if(
                in_state(GameState::ShowGame)
                    .and_then(resource_exists::<net::platform::Connection>),
            ),
        )
        .add_systems(
            Update,
            play::prepare.run_if(in_state(GameState::PrepareGame)),
        )
        .add_systems(
            Update,
            ui::show_game_panel.run_if(in_state(GameState::PrepareGame)),
        )
        .add_systems(
            Update,
            ui::show_resources_panel.run_if(in_state(GameState::PrepareGame)),
        )
        .add_systems(
            Update,
            net::platform::send_actions.run_if(in_state(GameState::ShowGame)),
        )
        .add_systems(
            Update,
            play::show_game.run_if(in_state(GameState::ShowGame)),
        )
        .add_systems(
            Update,
            play::handle_camera.run_if(in_state(GameState::ShowGame)),
        )
        .run();
}
