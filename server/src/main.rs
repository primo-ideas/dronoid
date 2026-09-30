#![forbid(unsafe_code)]

use clap::Parser;
use crossbeam_channel::TryRecvError::Disconnected;
use crossbeam_channel::{Receiver, Sender};
use std::io;
use std::net::SocketAddr;
use std::str::FromStr;
use thiserror::Error;
use tokio::net::TcpListener;
use tokio::signal;
use tokio_tungstenite::tungstenite;

use crate::game::run_game;
use crate::persistence::Database;
use crate::player::EnteringPlayer;
use crate::transport::run_transport;

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
    let transport_hdl = tokio::spawn(run_transport(
        database,
        tcp_listener,
        transport_stopper_rx,
        player_tx,
    ));
    let game_hdl = tokio::task::spawn_blocking(move || {
        run_game(rules, controls, transport_stopper_tx, player_rx)
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
    logger::init();
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

    run(rules, database, tcp_listener, controls).await?;

    anyhow::Ok(())
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use anyhow::bail;
    use dronoid_protocol::Action;
    use dronoid_protocol::Kind;
    use dronoid_protocol::Response;
    use dronoid_protocol::ServerMessage;
    use dronoid_protocol::State;
    use dronoid_protocol::{AuthenticationRequest, AuthenticationResponse, ClientMessage};
    use futures::SinkExt;
    use futures::StreamExt;
    use std::any;
    use std::f32;
    use std::net::SocketAddr;
    use std::time::Duration;
    use tempfile::NamedTempFile;
    use tokio::io::AsyncWriteExt;
    use tokio::time::Instant;
    use tokio::{net::TcpStream, task::JoinHandle};
    use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
    use tracing::info;
    use tungstenite::Message;

    use crate::Error;
    use crate::new_commands;
    use crate::persistence;
    use crate::run;
    use crate::{Commands, Rules};

    pub struct TestContext {
        pub commands: Commands,
        pub server_addr: SocketAddr,
        #[allow(unused)]
        pub rules: Rules,
        pub hdl: JoinHandle<Result<()>>,
    }

    impl TestContext {
        pub async fn setup(rules: Rules) -> anyhow::Result<Self> {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
            let server_addr = listener.local_addr()?;
            let (commands, controls) = new_commands();
            let rules_cln = rules.clone();
            let hdl = tokio::spawn(async move {
                run(
                    rules_cln,
                    persistence::Database::default(),
                    listener,
                    controls,
                )
                .await?;
                anyhow::Ok(())
            });
            Ok(Self {
                commands,
                server_addr,
                rules,
                hdl,
            })
        }

        pub async fn teardown(self) -> anyhow::Result<()> {
            self.commands.stop()?;
            self.hdl.await??;
            Ok(())
        }
    }

    #[allow(unused)]
    pub fn default_rules() -> Rules {
        Rules::default()
    }

    pub struct Client {
        stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        pub spawn_point: (f32, f32),
    }

    impl Client {
        pub async fn connect(addr: SocketAddr) -> Result<Self> {
            info!(sender = "Client", "Connecting to {addr}");
            let (stream, _) = tokio_tungstenite::connect_async(
                format!("ws://{}:{}/dronoid/ws", addr.ip(), addr.port()).as_str(),
            )
            .await
            .map_err(|err| Error::ClientConnectError(err))?;
            Ok(Self {
                stream,
                spawn_point: (0., 0.),
            })
        }

        pub async fn authenticated(addr: SocketAddr, player_name: &'static str) -> Result<Self> {
            let mut client = Self::connect(addr).await?;
            let response = client.authenticate(player_name).await?;
            if !response.result {
                bail!(Error::ClientFailedAuthentication(response.text));
            }
            Ok(client)
        }

        pub async fn authenticate(
            &mut self,
            player_name: &'static str,
        ) -> Result<AuthenticationResponse> {
            info!(sender = "Client", "Authenticating");
            self.stream
                .send(Message::Binary(
                    bson::serialize_to_vec(&ClientMessage::AuthenticationRequest(
                        AuthenticationRequest {
                            player_name: player_name.to_string(),
                        },
                    ))
                    .unwrap()
                    .into(),
                ))
                .await
                .map_err(|err| Error::ClientSendError(err))?;
            if let Some(maybe_binary) = self.stream.next().await {
                if let std::result::Result::Ok(Message::Binary(auth_resp_text)) = maybe_binary {
                    if let std::result::Result::Ok(response) =
                        bson::deserialize_from_slice::<ServerMessage>(
                            auth_resp_text.iter().as_slice(),
                        )
                    {
                        match response {
                            ServerMessage::Response(Response::AuthenticationResponse(response)) => {
                                self.spawn_point = response.spawn_point;
                                return Ok(response);
                            }
                            _ => {
                                bail!("Test client: read: Not a response or not auth response");
                            }
                        }
                    } else {
                        bail!("Test client: read: deserialization error");
                    }
                } else {
                    bail!(
                        "Test client: read: not okay: {}",
                        maybe_binary.err().unwrap()
                    );
                }
            } else {
                bail!("Test client: read: none");
            }
        }

        #[allow(unused)]
        pub async fn send_action(&mut self, action: Action) -> Result<()> {
            info!(sender = "Client", "Sending action");
            self.stream
                .send(Message::Binary(
                    bson::serialize_to_vec(&action).unwrap().into(),
                ))
                .await
                .map_err(|err| Error::ClientSendError(err))?;
            Ok(())
        }
        #[allow(unused)]
        pub async fn action(&mut self, action: Action) -> Result<Response> {
            self.send_action(action).await?;
            self.until_response().await
        }

        pub async fn next_message(&mut self) -> Result<ServerMessage> {
            let maybe_maybe_message = self.stream.next().await;
            if maybe_maybe_message.is_none() {
                bail!(Error::ClientReadError);
            }
            let message = maybe_maybe_message
                .unwrap()
                .map_err(|err| Error::RecvError(err))?;
            if let Message::Binary(text) = message {
                let server_message =
                    bson::deserialize_from_slice::<ServerMessage>(text.iter().as_slice())
                        .map_err(|_| Error::ClientReadError)?;
                return Ok(server_message);
            } else {
                bail!(Error::ClientReadError);
            }
        }
        #[allow(unused)]
        pub async fn until_response(&mut self) -> Result<Response> {
            loop {
                match self.next_message().await? {
                    ServerMessage::State(_) => continue,
                    ServerMessage::Response(response) => {
                        return Ok(response);
                    }
                }
            }
        }
        #[allow(unused)]
        pub async fn until_state(&mut self) -> Result<State> {
            loop {
                match self.next_message().await? {
                    ServerMessage::State(state) => {
                        return Ok(state);
                    }
                    ServerMessage::Response(_) => continue,
                }
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn network_01_tcp_connect() -> anyhow::Result<()> {
        let ctx = TestContext::setup(default_rules()).await?;
        tokio::net::TcpSocket::new_v4()?
            .connect(ctx.server_addr)
            .await?;
        ctx.teardown().await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn network_02_tcp_connect_and_close() -> anyhow::Result<()> {
        let ctx = TestContext::setup(default_rules()).await?;
        tokio::net::TcpSocket::new_v4()?
            .connect(ctx.server_addr)
            .await?
            .shutdown()
            .await?;
        ctx.teardown().await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn network_03_ws_upgrade() -> anyhow::Result<()> {
        let ctx = TestContext::setup(default_rules()).await?;
        tokio_tungstenite::connect_async(format!("ws://{}/dronoid/ws", ctx.server_addr))
            .await?
            .0
            .close(None)
            .await?;
        ctx.teardown().await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn auth_04_one_client_can_authenticate() -> anyhow::Result<()> {
        let ctx = TestContext::setup(default_rules()).await?;
        let mut client = Client::connect(ctx.server_addr).await?;
        let response = client.authenticate("Player").await?;
        assert!(response.result);
        assert_eq!("Welcome", response.text);
        assert!(0. != response.spawn_point.0);
        assert!(0. != response.spawn_point.1);
        ctx.teardown().await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn auth_05_two_clients_can_authenticate() -> anyhow::Result<()> {
        let ctx = TestContext::setup(default_rules()).await?;
        let mut client1 = Client::connect(ctx.server_addr).await?;
        let mut client2 = Client::connect(ctx.server_addr).await?;
        let response1 = client1.authenticate("Player1").await?;
        let response2 = client2.authenticate("Player2").await?;
        assert!(response2.result);
        assert_eq!("Welcome", response2.text);
        assert!(0. != response2.spawn_point.0);
        assert!(0. != response2.spawn_point.1);
        assert!(response1.spawn_point.0 != response2.spawn_point.0);
        assert!(response1.spawn_point.1 != response2.spawn_point.1);
        ctx.teardown().await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn auth_06_two_clients_same_name_fails() -> anyhow::Result<()> {
        let ctx = TestContext::setup(default_rules()).await?;
        let mut client1 = Client::connect(ctx.server_addr).await?;
        let mut client2 = Client::connect(ctx.server_addr).await?;
        let _ = client1.authenticate("Player").await?;
        let response = client2.authenticate("Player").await?;
        assert!(!response.result);
        assert_eq!("Already playing / name taken", response.text);
        assert_eq!((0., 0.), response.spawn_point);
        ctx.teardown().await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn gameplay_07_one_player_place_factory_in_zone() -> anyhow::Result<()> {
        let ctx = TestContext::setup(default_rules()).await?;
        let mut client = Client::authenticated(ctx.server_addr, "Player").await?;
        client
            .send_action(Action::PlaceFactory(client.spawn_point))
            .await?;
        let response = client.until_response().await?;
        if let Response::PlaceFactory { result } = response {
            assert!(result);
        } else {
            bail!("Not a place factory response");
        }
        ctx.teardown().await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn gameplay_08_one_player_place_factory_one_response_only() -> anyhow::Result<()> {
        let ctx = TestContext::setup(default_rules()).await?;
        let mut client = Client::authenticated(ctx.server_addr, "Player").await?;
        client
            .send_action(Action::PlaceFactory(client.spawn_point))
            .await?;
        client.until_response().await?;
        assert!(
            tokio::time::timeout(Duration::from_secs_f64(0.5), client.until_response())
                .await
                .is_err()
        );
        ctx.teardown().await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn gameplay_09_one_player_place_factory_out_of_zone_fails() -> anyhow::Result<()> {
        let ctx = TestContext::setup(default_rules()).await?;
        let mut client = Client::authenticated(ctx.server_addr, "Player").await?;
        let mut place_position = client.spawn_point;
        place_position.0 += 1000.;
        client
            .send_action(Action::PlaceFactory(place_position))
            .await?;
        let response = client.until_response().await?;
        if let Response::PlaceFactory { result } = response {
            assert!(!result);
        } else {
            bail!("Not a place factory response");
        }
        ctx.teardown().await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn gameplay_10_two_players_cant_place_factory_on_other_zone() -> anyhow::Result<()> {
        let ctx = TestContext::setup(default_rules()).await?;
        let mut client1 = Client::authenticated(ctx.server_addr, "Player1").await?;
        let mut client2 = Client::authenticated(ctx.server_addr, "Player2").await?;
        client1
            .send_action(Action::PlaceFactory(client2.spawn_point))
            .await?;
        let response1 = client1.until_response().await?;
        client2
            .send_action(Action::PlaceFactory(client1.spawn_point))
            .await?;
        let response2 = client2.until_response().await?;

        if let Response::PlaceFactory { result } = response1 {
            assert!(!result);
        } else {
            bail!("Not a place factory response");
        }
        if let Response::PlaceFactory { result } = response2 {
            assert!(!result);
        } else {
            bail!("Not a place factory response");
        }
        ctx.teardown().await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn gameplay_11_one_player_receive_first_state() -> anyhow::Result<()> {
        let ctx = TestContext::setup(default_rules()).await?;
        let mut client = Client::authenticated(ctx.server_addr, "Player").await?;
        let first_state = client.until_state().await?;
        assert!(first_state.entities_in_zone.len() > 0);
        assert_eq!(
            1,
            first_state
                .entities_in_zone
                .iter()
                .fold(0, |a, x| { if x.kind == Kind::Spawn { a + 1 } else { a } })
        );
        let entity = first_state
            .entities_in_zone
            .iter()
            .find(|x| x.kind == Kind::Spawn)
            .unwrap();
        assert_eq!(entity.pos.0, client.spawn_point.0);
        assert_eq!(entity.pos.1, client.spawn_point.1);
        assert_eq!(entity.kind, Kind::Spawn);
        ctx.teardown().await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn gameplay_12_one_player_receive_two_states() -> anyhow::Result<()> {
        let ctx = TestContext::setup(default_rules()).await?;
        let mut client = Client::authenticated(ctx.server_addr, "Player").await?;
        client.until_state().await?;
        let second_state = client.until_state().await?;
        assert!(second_state.entities_in_zone.len() > 0);
        assert_eq!(
            1,
            second_state
                .entities_in_zone
                .iter()
                .fold(0, |a, x| { if x.kind == Kind::Spawn { a + 1 } else { a } })
        );
        let entity = second_state
            .entities_in_zone
            .iter()
            .find(|x| x.kind == Kind::Spawn)
            .unwrap();
        assert_eq!(entity.pos.0, client.spawn_point.0);
        assert_eq!(entity.pos.1, client.spawn_point.1);
        assert!(
            10 < second_state.entities_in_zone.iter().fold(0, |a, x| {
                if x.kind == Kind::Mineral { a + 1 } else { a }
            })
        );
        ctx.teardown().await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn gameplay_13_two_players_receive_first_state() -> anyhow::Result<()> {
        let ctx = TestContext::setup(default_rules()).await?;
        let mut client1 = Client::authenticated(ctx.server_addr, "Player1").await?;
        let mut client2 = Client::authenticated(ctx.server_addr, "Player2").await?;
        let first_state1 = client1.until_state().await?;
        let first_state2 = client2.until_state().await?;

        assert!(first_state1.entities_in_zone.len() > 0);
        let entity1 = first_state1
            .entities_in_zone
            .iter()
            .find(|x| x.kind == Kind::Spawn)
            .unwrap();
        assert_eq!(entity1.pos.0, client1.spawn_point.0);
        assert_eq!(entity1.pos.1, client1.spawn_point.1);

        assert!(first_state2.entities_in_zone.len() > 0);
        let entity2 = first_state2
            .entities_in_zone
            .iter()
            .find(|x| x.kind == Kind::Spawn)
            .unwrap();
        assert_eq!(entity2.pos.0, client2.spawn_point.0);
        assert_eq!(entity2.pos.1, client2.spawn_point.1);

        ctx.teardown().await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn gameplay_14_place_factory_circle() -> anyhow::Result<()> {
        let rules = Rules {
            starting_minerals: 10000,
            ..Default::default()
        };
        let ctx = TestContext::setup(rules.clone()).await?;
        let mut client = Client::authenticated(ctx.server_addr, "Player").await?;
        let mut angle = -f32::consts::PI;
        while angle < f32::consts::PI {
            let response = tokio::time::timeout(
                Duration::from_secs_f32(1.),
                client.action(Action::PlaceFactory((
                    client.spawn_point.0 + rules.factory_extension * (angle.cos()),
                    client.spawn_point.1 + rules.factory_extension * (angle.sin()),
                ))),
            )
            .await??;
            if let Response::PlaceFactory { result } = response {
                assert!(result);
            } else {
                bail!("Not the expected PlaceFactory response");
            }
            angle += f32::consts::TAU / 10.;
        }
        while angle < f32::consts::PI {
            let response = tokio::time::timeout(
                Duration::from_secs_f32(1.),
                client.action(Action::PlaceFactory((
                    client.spawn_point.0 + (rules.factory_extension + f32::EPSILON) * (angle).cos(),
                    client.spawn_point.1 + (rules.factory_extension + f32::EPSILON) * (angle).sin(),
                ))),
            )
            .await??;
            if let Response::PlaceFactory { result } = response {
                assert!(!result);
            } else {
                bail!("Not the expected PlaceFactory response");
            }
            angle += f32::consts::TAU / 10.;
        }
        ctx.teardown().await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ping_01_twenty_messages() -> anyhow::Result<()> {
        let ctx = TestContext::setup(default_rules()).await?;
        let mut client = Client::authenticated(ctx.server_addr, "Player1").await?;
        let mut now = Instant::now();
        let mut times = Vec::<f32>::new();

        for _ in 1..20 {
            client.next_message().await?;
            let new_now = Instant::now();
            times.push((new_now - now).as_secs_f32());
            now = new_now;
        }

        let average_ping: f32 = times.iter().sum::<f32>() / times.len() as f32;
        assert!(average_ping > ctx.rules.tick_duration - 0.05);
        assert!(average_ping < ctx.rules.tick_duration + 0.05);

        ctx.teardown().await
    }

    #[test]
    fn persistence_01_add_get_player() -> anyhow::Result<()> {
        let tmp_db_path = NamedTempFile::new()?;
        let db = persistence::Database::from_path(tmp_db_path.path().to_str().unwrap());

        db.add_player_entry(name, spawn_point)
        anyhow::Ok(())
    }
}
