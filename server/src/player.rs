use anyhow::*;
use crossbeam_channel::{Receiver, Sender};
use std::collections::HashMap;
use std::net::SocketAddr;

use crate::protocol::Action;
use crate::protocol::ServerMessage;

pub struct OnlinePlayer {
    pub messages: Vec<ServerMessage>,
    message_sender: Sender<ServerMessage>,
    pub action_receiver: Receiver<Action>,
    pub _name: String,
    pub minerals_cnt: u32,
    pub owned_factories: HashMap<u32, bevy_ecs::entity::Entity>,
    pub to_kick: bool,
}

impl OnlinePlayer {
    pub fn new(
        message_sender: Sender<ServerMessage>,
        action_receiver: Receiver<Action>,
        name: String,
        minerals_cnt: u32,
    ) -> Self {
        Self {
            _name: name,
            action_receiver,
            message_sender,
            messages: Vec::new(),
            minerals_cnt,
            owned_factories: HashMap::new(),
            to_kick: false,
        }
    }
    pub fn flush_messages(&mut self) -> anyhow::Result<()> {
        for message in self.messages.drain(0..) {
            self.message_sender
                .send(message)
                .map_err(|_| anyhow!("Flush error"))?;
        }
        Ok(())
    }
}

pub struct EnteringPlayer {
    pub message_sender: Sender<ServerMessage>,
    pub action_receiver: Receiver<Action>,
    pub id: u32,
    pub name: String,
    pub spawn_point: (i32, i32),
    pub _addr: SocketAddr,
}
