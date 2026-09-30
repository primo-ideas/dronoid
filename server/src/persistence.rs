use rusqlite::{Connection, OpenFlags};

use crate::helper::gen_id;

#[derive(Clone)]
pub struct PlayerEntry {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) spawn_point: (f32, f32),
}

pub struct Database {
    conn: rusqlite::Connection,
    players: Vec<PlayerEntry>,
}

impl Database {
    pub fn from_path(path: &str) -> Database {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_CREATE).unwrap();
        conn.execute(
            "CREATE TABLE player (
            id    INTEGER PRIMARY KEY,
            nickname  TEXT NOT NULL,
            spawn_x  INTEGER,
            spawn_y  INTEGER,
            mineral_cnt INTEGER,
        )",
            (),
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE building (
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

        conn.execute(
            "CREATE TABLE mineral (
            id    INTEGER PRIMARY KEY,
            pos_x  INTEGER,
            pos_y  INTEGER,
        )",
            (),
        )
        .unwrap();

        Database {
            conn,
            players: Vec::new(),
        }
    }

    pub async fn get_player_entry(&self, name: &String) -> Option<PlayerEntry> {
        self.players.iter().find(|x| x.name == *name).cloned()
    }

    pub async fn add_player_entry(&mut self, name: String, spawn_point: (f32, f32)) -> PlayerEntry {
        let player_entry = PlayerEntry {
            id: gen_id(),
            spawn_point,
            name,
        };
        self.players.push(player_entry.clone());
        player_entry
    }
}
