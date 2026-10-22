use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tungstenite::Utf8Bytes;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
#[serde(rename_all = "lowercase")]
pub enum Action {
    PlaceFactory((i32, i32)),
    ControlFactory(FactoryControl),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Eq, Hash)]
pub enum Kind {
    Mineral,
    Dronoid,
    Factory,
    Beacon,
}

#[derive(Serialize, Deserialize)]
pub struct Entity {
    pub pos: (f32, f32),
    pub kind: Kind,
    pub id: u32,
}

#[derive(Serialize, Deserialize, Default)]
pub struct PlayerInfo {
    pub minerals_cnt: u32,
}

#[derive(Serialize, Deserialize, Default)]
pub struct StateMessage {
    pub visible_minerals: Vec<Entity>,
    pub player_info: PlayerInfo,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseMessage {
    Authentication {
        result: bool,
        message: String,
        spawn: (i32, i32),
    },
    PlaceFactory {
        result: bool,
        message: String,
    },
    ControlFactory {
        result: bool,
    },
}

#[allow(unused)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClientMessage {
    AuthenticationRequest { nickname: String },
    Action(Action),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]

pub enum ServerMessage {
    Response(ResponseMessage),
    State(StateMessage),
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
