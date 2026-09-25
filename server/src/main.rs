#![forbid(unsafe_code)]

use clap::Parser;
use crossbeam_channel::TryRecvError::Disconnected;
use crossbeam_channel::{Receiver, Sender};
use std::io;
use thiserror::Error;
use tokio::net::TcpListener;
use tokio::signal;
use tokio_tungstenite::tungstenite;

use crate::persistence::Database;
use crate::player::EnteringPlayer;

mod component;
mod game;
mod helper;
mod logger;
mod persistence;
mod player;
mod resource;
mod system;
mod transport;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Network could not be initialized: {0}")]
    NetworkInitError(io::Error),
    #[error("Did not stop properly: {0}")]
    StopError(u8),
    #[error("Could not flush state to transport")]
    FlushError,
    #[error("Failed to join session: {0}")]
    WaitError(tokio::task::JoinError),
    #[error("No message received or it was of an unexpected type")]
    UnexpectedOrNoMessage,
    #[error("Client could not connect: {0}")]
    ClientConnectError(tungstenite::Error),
    #[error("Client could not authentication: {0}")]
    ClientFailedAuthentication(String),
    #[error("Client could not send data: {0}")]
    ClientSendError(tungstenite::Error),
    #[error("Client read error")]
    ClientReadError,
    #[error("Client could not receive data: {0}")]
    RecvError(tungstenite::Error),
    #[error("Stopper channel failed: {0}")]
    StopperError(crossbeam_channel::SendError<Stop>),
    #[error("Unexpected error: {0}")]
    UnexpectedError(&'static str),
    #[error("Transport error")]
    TransportError,
    #[error("Not an action")]
    NotAnAction,
    #[error("Unknown error")]
    UnknownError,
    #[error("ser/de failed: {0}")]
    Serde(bson::error::Error),
}

#[derive(Clone, Debug)]
pub struct ZoneExtensions {
    pub factory: f32,
}

impl Default for ZoneExtensions {
    fn default() -> Self {
        Self {
            factory: dronoid_protocol::FACTORY_EXTENSION,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rules {
    pub terrain_scale: f32,
    pub mineral_threshold: f32,
    pub tick_duration: f32,
    pub starting_minerals: u32,
    pub terrain_seed: u32,
    pub factory_extension: f32,
    pub factory_cost: u32,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            terrain_scale: dronoid_protocol::TERRAIN_SCALE,
            mineral_threshold: dronoid_protocol::MINERAL_THRESHOLD,
            tick_duration: dronoid_protocol::TICK_DURATION,
            starting_minerals: dronoid_protocol::STARTING_MINERALS,
            terrain_seed: dronoid_protocol::TERRAIN_SEED,
            factory_extension: dronoid_protocol::FACTORY_EXTENSION,
            factory_cost: dronoid_protocol::FACTORY_COST,
        }
    }
}

pub enum Stop {
    Normal,
}

pub struct Commands {
    stopper: Sender<Stop>,
}

impl Commands {
    pub fn stop(&self) -> Result<()> {
        self.stopper
            .send(Stop::Normal)
            .map_err(Error::StopperError)?;
        Ok(())
    }
}

pub struct Controls {
    stopper: Receiver<Stop>,
}

impl Controls {
    pub fn stopped(&self) -> bool {
        match self.stopper.try_recv() {
            Err(Disconnected) => true,
            Ok(_) => true,
            _ => false,
        }
    }
}

pub async fn run(
    rules: Rules,
    database: Database,
    tcp_listener: TcpListener,
    controls: Controls,
) -> Result<()> {
    let (player_tx, player_rx) = crossbeam_channel::bounded::<EnteringPlayer>(1000);
    let (transport_stopper_tx, transport_stopper_rx) = crossbeam_channel::bounded::<()>(1);
    let transport_hdl = tokio::spawn(transport::run(
        database,
        tcp_listener,
        transport_stopper_rx,
        player_tx,
    ));
    let game_hdl = tokio::task::spawn_blocking(move || {
        game::run(rules, controls, transport_stopper_tx, player_rx)
    });
    let _ = tokio::join!(transport_hdl, game_hdl);
    Ok(())
}

pub fn new_commands() -> (Commands, Controls) {
    let (tx, rx) = crossbeam_channel::bounded(1);
    (Commands { stopper: tx }, Controls { stopper: rx })
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(default_value_t = 8080)]
    port: u16,
    #[arg(long, default_value_t = dronoid_protocol::TERRAIN_SCALE)]
    terrain_scale: f32,
    #[arg(long, default_value_t = dronoid_protocol::MINERAL_THRESHOLD)]
    mineral_threshold: f32,
    #[arg(long, default_value_t = dronoid_protocol::TICK_DURATION)]
    tick_duration: f32,
    #[arg(long, default_value_t = dronoid_protocol::STARTING_MINERALS)]
    starting_minerals: u32,
    #[arg(long, default_value_t = dronoid_protocol::TERRAIN_SEED)]
    terrain_seed: u32,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    // dronoid_logger::init();
    let tcp_listener = TcpListener::bind(SocketAddr::from_str(
        format!("127.0.0.1:{}", args.port).as_str(),
    )?)
    .await?;
    let rules = Rules {
        terrain_scale: args.terrain_scale,
        mineral_threshold: args.mineral_threshold,
        tick_duration: args.tick_duration,
        starting_minerals: args.starting_minerals,
        terrain_seed: args.terrain_seed,
        ..Default::default()
    };
    let database = persistence::Database::default();
    let (commands, controls) = new_commands();
    tokio::spawn(async move {
        #[cfg(target_os = "windows")]
        let mut ctrl_close = signal::windows::ctrl_close().unwrap();
        #[cfg(target_os = "windows")]
        let mut ctrl_logoff = signal::windows::ctrl_logoff().unwrap();
        #[cfg(target_os = "windows")]
        let mut ctrl_shutdown = signal::windows::ctrl_shutdown().unwrap();
        #[cfg(target_os = "windows")]
        tokio::select! {
            _ = signal::ctrl_c() => {}
            _ = ctrl_close.recv() => {}
            _ = ctrl_logoff.recv() => {}
            _ = ctrl_shutdown.recv() => {}
        }
        #[cfg(target_os = "linux")]
        let _ = signal::ctrl_c().await;
        let _ = commands.stop();
    });

    dronoid_server::run(rules, database, tcp_listener, controls).await?;

    anyhow::Ok(())
}
