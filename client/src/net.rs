use bevy::prelude::*;
use dronoid_protocol::{AuthenticationRequest, ClientMessage, ServerMessage};
use std::{net::TcpStream, str::FromStr};
use tungstenite::{Bytes, stream::MaybeTlsStream};

use crate::{
    CloseConnectionMessage, GameState, ProgramArgs,
    game::SpawnPoint,
    ui::{InfoMessage, PlayerName},
};

#[derive(Message)]
pub struct StateMessage(pub dronoid_protocol::State);

#[derive(Message)]
pub struct ActionMessage(pub dronoid_protocol::Action);

#[derive(Resource)]
pub struct Connection(pub tungstenite::WebSocket<MaybeTlsStream<TcpStream>>);

impl Connection {
    pub fn new(websocket: tungstenite::WebSocket<MaybeTlsStream<TcpStream>>) -> Self {
        Self(websocket)
    }
}

pub fn plugin(app: &mut App) {
    app.add_message::<StateMessage>();
    app.add_message::<ActionMessage>();
    app.add_systems(Update, connect.run_if(in_state(GameState::Connect)));
    app.add_systems(
        Update,
        authenticate_send_request.run_if(in_state(GameState::AuthenticateSendRequest)),
    );
    app.add_systems(
        Update,
        authenticate_wait_response.run_if(in_state(GameState::AuthenticateWaitResponse)),
    );
    app.add_systems(
        Update,
        read_server_messages.run_if(in_state(GameState::Play)),
    );
    app.add_systems(Update, send_actions.run_if(in_state(GameState::Play)));
    app.add_systems(Update, close_connection.run_if(in_state(GameState::Play)));
}

pub fn connect(
    mut info_label: MessageWriter<InfoMessage>,
    mut next_state: ResMut<NextState<GameState>>,
    program_options: Res<ProgramArgs>,
    mut commands: Commands,
) {
    let mut protocol = "wss";
    if let true = program_options.no_tls {
        protocol = "ws";
    }
    let addr = format!(
        "{}://{}:{}/{}",
        protocol, program_options.hostname, program_options.port, program_options.route
    )
    .to_string();
    info_label.write(InfoMessage(addr.clone()));
    if let Ok(uri) = tungstenite::http::Uri::from_str(addr.as_str()) {
        match tungstenite::connect(tungstenite::ClientRequestBuilder::new(uri)) {
            Ok((websocket, _)) => {
                commands.insert_resource(Connection::new(websocket));
                info_label.write(InfoMessage("Sending auth request".to_string()));
                next_state.set(GameState::AuthenticateSendRequest);
            }
            Err(err) => {
                info_label.write(InfoMessage(
                    format!("Connection failed: {}", err).to_string(),
                ));
                next_state.set(GameState::Welcome);
            }
        }
    } else {
        info_label.write(InfoMessage("URI build failed".to_string()));
        next_state.set(GameState::Welcome);
    }
}

pub fn authenticate_send_request(
    mut info_label: MessageWriter<InfoMessage>,
    mut state: ResMut<NextState<GameState>>,
    mut connection: ResMut<Connection>,
    player_name: Res<PlayerName>,
) {
    let maybe_auth_request = bson::serialize_to_vec(&ClientMessage::AuthenticationRequest(
        AuthenticationRequest {
            player_name: player_name.0.clone(),
        },
    ));
    if maybe_auth_request.is_err() {
        info_label.write(InfoMessage(format!(
            "Auth request serialization error: {}",
            maybe_auth_request.err().unwrap()
        )));
        state.set(GameState::Welcome);
        return;
    }
    let auth_request = maybe_auth_request.unwrap();
    let result = connection
        .0
        .write(tungstenite::Message::Binary(Bytes::from(auth_request)));

    if result.is_err() {
        info_label.write(InfoMessage(format!(
            "Auth request send error: {}",
            result.err().unwrap()
        )));
        state.set(GameState::Welcome);
        return;
    }

    let result = connection.0.flush();
    if result.is_err() {
        info_label.write(InfoMessage(format!(
            "Auth request send error: {}",
            result.err().unwrap()
        )));
        state.set(GameState::Welcome);
        return;
    }
    info_label.write(InfoMessage("Waiting auth response".to_string().to_string()));
    state.set(GameState::AuthenticateWaitResponse);
}

pub fn authenticate_wait_response(
    mut info_label: MessageWriter<InfoMessage>,
    mut state: ResMut<NextState<GameState>>,
    mut connection: ResMut<Connection>,
    mut spawn_point: ResMut<SpawnPoint>,
) {
    let maybe_response = connection.0.read();
    if maybe_response.is_err() {
        info_label.write(InfoMessage(format!(
            "Auth response read error: {}",
            maybe_response.err().unwrap()
        )));
        state.set(GameState::Welcome);
        return;
    }

    let response = maybe_response.unwrap();
    if let tungstenite::Message::Binary(binary) = response {
        let maybe_auth_response = bson::deserialize_from_slice::<dronoid_protocol::ServerMessage>(
            binary.iter().as_slice(),
        );
        if maybe_auth_response.is_err() {
            info_label.write(InfoMessage(format!(
                "Auth response deserialization error: {}",
                maybe_auth_response.err().unwrap()
            )));
            state.set(GameState::Welcome);
            return;
        }
        match maybe_auth_response.unwrap() {
            dronoid_protocol::ServerMessage::Response(
                dronoid_protocol::Response::AuthenticationResponse(auth_response),
            ) => {
                if !auth_response.result {
                    info_label.write(InfoMessage(format!(
                        "Server declined authentication: {}",
                        auth_response.text
                    )));
                    state.set(GameState::Welcome);
                    return;
                }
                spawn_point.0 = auth_response.spawn_point;
                info_label.write(InfoMessage("".to_string()));
                match connection.0.get_mut() {
                    MaybeTlsStream::Plain(s) => s.set_nonblocking(true).unwrap(),
                    MaybeTlsStream::Rustls(s) => s.sock.set_nonblocking(true).unwrap(),
                    _ => {}
                }

                state.set(GameState::PrepareGame);
            }
            _ => {
                std::process::abort();
            }
        }
    } else {
        info_label.write(InfoMessage(
            "Unexpected non binary auth response".to_string(),
        ));
        state.set(GameState::Welcome);
    }
}

pub fn read_server_messages(
    mut connection: ResMut<Connection>,
    mut server_messages: MessageWriter<StateMessage>,
) {
    while let Ok(tungstenite::Message::Binary(message)) = connection.0.read() {
        let server_message = bson::deserialize_from_slice::<dronoid_protocol::ServerMessage>(
            &message.iter().as_slice(),
        )
        .unwrap();

        if let ServerMessage::State(state) = server_message {
            server_messages.write(StateMessage(state));
        }
    }
}

pub fn send_actions(mut connection: ResMut<Connection>, mut actions: MessageReader<ActionMessage>) {
    for action in actions.read() {
        let binary_to_send = bson::serialize_to_vec(&action.0).unwrap();
        connection
            .0
            .write(tungstenite::Message::Binary(Bytes::from(binary_to_send)))
            .unwrap();
    }
    connection.0.flush().unwrap();
}

pub fn close_connection(
    mut connection: ResMut<Connection>,
    mut leave_messages: MessageReader<CloseConnectionMessage>,
) {
    if leave_messages.read().count() > 0 {
        leave_messages.clear();
        let _ = connection.0.close(None);
        let _ = connection.0.flush();
    }
}
