use rusqlite::Connection;

use crate::helper::gen_id;

#[derive(Clone)]
pub struct PlayerEntry {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) spawn_point: (i32, i32),
}

pub struct Database {
    _conn: rusqlite::Connection,
    players: Vec<PlayerEntry>,
}

impl Database {
    pub fn from_path(maybe_path: Option<&str>) -> Database {
        let _conn = if let Some(path) = maybe_path {
            Connection::open(path).unwrap()
        } else {
            Connection::open_in_memory().unwrap()
        };
        _conn
            .execute(
                "CREATE TABLE IF NOT EXISTS player (
            id    INTEGER PRIMARY KEY,
            nickname  TEXT NOT NULL,
            spawn_x  INTEGER,
            spawn_y  INTEGER,
            mineral_cnt INTEGER
        )",
                (),
            )
            .unwrap();

        _conn
            .execute(
                "CREATE TABLE IF NOT EXISTS building (
            id    INTEGER PRIMARY KEY,
            type  INTEGER,
            pos_x  INTEGER,
            pos_y  INTEGER,
            player_id INTEGER,
            FOREIGN KEY(player_id) REFERENCES player(id)
        )",
                (),
            )
            .unwrap();

        _conn
            .execute(
                "CREATE TABLE IF NOT EXISTS mineral (
            id    INTEGER PRIMARY KEY,
            pos_x  INTEGER,
            pos_y  INTEGER
        )",
                (),
            )
            .unwrap();

        Database {
            _conn,
            players: Vec::new(),
        }
    }

    pub fn get_player_entry(&self, name: &String) -> Option<PlayerEntry> {
        self.players.iter().find(|x| x.name == *name).cloned()
    }

    pub fn add_player_entry(&mut self, name: String, spawn_point: (i32, i32)) -> PlayerEntry {
        let player_entry = PlayerEntry {
            id: gen_id(),
            spawn_point,
            name,
        };
        self.players.push(player_entry.clone());
        player_entry
    }
}
