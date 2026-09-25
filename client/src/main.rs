#![forbid(unsafe_code)]

use bevy::{asset::AssetMetaCheck, input_focus::tab_navigation::TabNavigationPlugin, prelude::*};
use clap::Parser;

mod game;
mod net;
mod ui;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value_t = "127.0.0.1".to_string(), env = "DRONOID_CLIENT_HOSTNAME")]
    hostname: String,
    #[arg(long, default_value_t = 443, env = "DRONOID_CLIENT_PORT")]
    port: u16,
    #[arg(long, default_value_t = false, env = "DRONOID_CLIENT_TLS")]
    no_tls: bool,
    #[arg(long, default_value_t = "dronoid/ws".to_string(), env = "DRONOID_CLIENT_ROUTE")]
    route: String,
}

fn main() -> () {
    let args = Args::parse();

    App::new()
        .add_plugins((
            DefaultPlugins
                .build()
                // .disable::<LogPlugin>()
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
        // .insert_resource(ClearColor(Color::srgb(0., 0., 0.)))
        // .insert_resource(Entities::default())
        // .insert_resource(GameSprites::default())
        // .insert_resource(PlayerName(args.player_name))
        // .insert_resource(SpawnPoint::default())
        // .insert_resource(ProgramOptions {
        //     route: args.route,
        //     tls: !args.no_tls,
        //     port: args.port,
        //     hostname: args.hostname,
        // })
        // .init_state::<GameState>()
        // .init_state::<PlayState>()
        // .add_message::<ServerMessage>()
        // .add_message::<ActionMessage>()
        // .add_message::<InfoMessage>()
        // .add_message::<LeaveMessage>()
        // .add_systems(PreStartup, platform::setup_display)
        // .add_systems(Startup, setup_sprites)
        // .add_systems(Startup, ui::setup::ui_camera)
        // .add_systems(Startup, ui::setup::connect_page)
        // .add_systems(Startup, ui::setup::game_panel)
        // .add_systems(Startup, ui::setup::resources_panel)
        // .add_systems(Startup, ui::setup::leave_game_button)
        // .add_systems(Update, ui::info_label)
        // .add_systems(Update, ui::handle::buttons)
        // .add_systems(
        //     Update,
        //     ui::handle::place_factory_button
        //         .run_if(in_state(GameState::ShowGame).and_eager(in_state(PlayState::Idle))),
        // )
        // .add_systems(
        //     Update,
        //     ui::handle::placing_factory.run_if(
        //         in_state(GameState::ShowGame).and_eager(in_state(PlayState::PlacingFactory)),
        //     ),
        // )
        // .add_systems(
        //     Update,
        //     ui::show_connect_page.run_if(in_state(GameState::ShowConnectPage)),
        // )
        // .add_systems(
        //     Update,
        //     ui::handle::connect_button.run_if(in_state(GameState::HandleConnectPage)),
        // )
        // .add_systems(
        //     Update,
        //     ui::handle::leave_game_button.run_if(in_state(GameState::ShowGame)),
        // )
        // .add_systems(
        //     Update,
        //     net::platform::connect.run_if(in_state(GameState::Connect)),
        // )
        // .add_systems(
        //     Update,
        //     net::platform::leave.run_if(in_state(GameState::ShowGame)),
        // )
        // .add_systems(
        //     Update,
        //     net::platform::authenticate_send_request.run_if(
        //         in_state(GameState::AuthenticateSendRequest)
        //             .and_then(resource_exists::<net::platform::Connection>),
        //     ),
        // )
        // .add_systems(
        //     Update,
        //     net::platform::authenticate_wait_response.run_if(
        //         in_state(GameState::AuthenticateWaitResponse)
        //             .and_then(resource_exists::<net::platform::Connection>),
        //     ),
        // )
        // .add_systems(
        //     Update,
        //     net::platform::read_server_messages.run_if(
        //         in_state(GameState::ShowGame)
        //             .and_then(resource_exists::<net::platform::Connection>),
        //     ),
        // )
        // .add_systems(
        //     Update,
        //     play::prepare.run_if(in_state(GameState::PrepareGame)),
        // )
        // .add_systems(
        //     Update,
        //     ui::show_game_panel.run_if(in_state(GameState::PrepareGame)),
        // )
        // .add_systems(
        //     Update,
        //     ui::show_resources_panel.run_if(in_state(GameState::PrepareGame)),
        // )
        // .add_systems(
        //     Update,
        //     ui::show_leave_game_button.run_if(in_state(GameState::PrepareGame)),
        // )
        // .add_systems(
        //     Update,
        //     net::platform::send_actions.run_if(in_state(GameState::ShowGame)),
        // )
        // .add_systems(
        //     Update,
        //     play::show_game.run_if(in_state(GameState::ShowGame)),
        // )
        // .add_systems(
        //     Update,
        //     play::handle_camera.run_if(in_state(GameState::ShowGame)),
        // )
        .run();
}
