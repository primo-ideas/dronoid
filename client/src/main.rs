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

#[derive(Resource)]
pub struct ProgramArgs {
    pub hostname: String,
    pub port: u16,
    pub no_tls: bool,
    pub route: String,
}

fn main() -> () {
    let args = Args::parse();

    App::new()
        .add_plugins((
            DefaultPlugins
                .build()
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
        .insert_resource(ProgramArgs {
            hostname: args.hostname,
            port: args.port,
            route: args.route,
            no_tls: args.no_tls,
        })
        .add_plugins((game::plugin, net::plugin, ui::plugin))
        .run();
}
