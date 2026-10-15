use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tungstenite::Utf8Bytes;

// impl AuthenticationResponse {
//     pub fn new(result: bool, text: &'static str, spawn_point: (f32, f32)) -> Self {
//         Self {
//             result,
//             text: text.to_string(),
//             spawn_point,
//         }
//     }

//     pub fn already_playing() -> Self {
//         Self::new(false, "Already playing / name taken", (0., 0.))
//     }

//     pub fn invalid_name() -> Self {
//         Self::new(false, "Invalid name", (0., 0.))
//     }

//     pub fn welcome(spawn_point: (f32, f32)) -> Self {
//         Self::new(true, "Welcome", spawn_point)
//     }
// }

#[derive(Serialize, Deserialize)]
pub enum FactoryOrder {
    SetAutoSpawn(bool),
    ManualSpawn,
}

#[derive(Serialize, Deserialize)]
pub struct FactoryControl {
    pub id: u32,
    pub order: FactoryOrder,
}

#[derive(Serialize, Deserialize)]
pub enum Action {
    PlaceFactory((i32, i32)),
    ControlFactory(FactoryControl),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Eq, Hash)]
pub enum Kind {
    Mineral,
    Factory,
    Spawn,
    Dronoid,
}

#[derive(Serialize, Deserialize)]
pub struct Entity {
    pub pos: (f32, f32),
    pub kind: Kind,
    pub id: u32,
}

#[derive(Serialize, Deserialize, Default)]
pub struct State {
    pub entities_in_zone: Vec<Entity>,
    pub minerals_cnt: u32,
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    Authentication {
        result: bool,
        message: String,
        spawn: (i32, i32),
    },
    PlaceFactory {
        result: bool,
    },
    ControlFactory {
        result: bool,
    },
}

#[allow(unused)]
#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    AuthenticationRequest { nickname: String },
    Action(Action),
}

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    Response(Response),
    State(State),
}

pub fn is_name_valid(player_name: &str) -> bool {
    !player_name.is_empty()
}

pub(crate) fn serialize<T>(to_serialize: T) -> Utf8Bytes
where
    T: Serialize,
{
    serde_json::to_string(&to_serialize).unwrap().into()
}

pub fn deserialize<T>(to_deserialize: Utf8Bytes) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    let str = to_deserialize.as_str();
    let val = serde_json::from_str(str)?;
    Ok(val)
}
