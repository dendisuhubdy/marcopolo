// Copyright 2019 MarcoPolo Protocol Authors.
// This file is part of MarcoPolo Protocol.

// MarcoPolo Protocol is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// MarcoPolo Protocol is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with MarcoPolo Protocol.  If not, see <http://www.gnu.org/licenses/>.

extern crate crossbeam;
extern crate core;
// use crossbeam::{crossbeam_channel};
use crossbeam::crossbeam_channel::{bounded, select, Receiver, RecvError, Sender};
use std::collections::HashMap;
use core::{block::Block};

pub struct RegisterItem<M> (pub String, pub Sender<Receiver<M>>);
pub type EventRegister<M> = Sender<RegisterItem<M>>;

impl<M> RegisterItem<M> {
    pub fn call(sender: &Sender<RegisterItem<M>>, arguments: String) -> Option<Receiver<M>> {
        let (responder, response) = crossbeam_channel::bounded(ONESHOT_CHANNEL_SIZE);
        let _ = sender.send(RegisterItem(responder,arguments));
        response.recv().ok()
    }
}

#[derive(Clone)]
pub struct EventHandler {
    new_block_register: EventRegister<Block>,
    new_block_notifier: Sender<Block>,
}
impl EventHandler {
    pub fn subscribe_new_block<S: ToString>(&self, name: S) -> Receiver<Block> {
        RegisterItem::call(&self.new_block_register, name.to_string())
            .expect("Subscribe new block should be OK")
    }
    pub fn notify_new_block(&self, b: Block) {
        let _ = self.new_block_notifier.send(b);
    }
}
pub struct EventService {
    new_block_subscribers: HashMap<String, Sender<Block>>,
}

impl EventService {
    pub fn new() -> Self {
        Self {
            new_block_subscribers :HashMap::default(),
        }
    }
    fn handle_register_new_block(
        &mut self,
        msg: Result<RegisterItem<Block>, RecvError>,
    ) {
        match msg {
            Ok(RegisterItem (
                name,
                responder,
            )) => {
                debug!("Register new_block {:?}", name);
                let (sender, receiver) = bounded::<Block>(NOTIFY_CHANNEL_SIZE);
                self.new_block_subscribers.insert(name, sender);
                let _ = responder.send(receiver);
            }
            _ => debug!("Register new_block channel is closed"),
        }
    }

    fn handle_notify_new_block(&mut self, msg: Result<Block, RecvError>) {
        match msg {
            Ok(block) => {
                trace!("event new block {:?}", block);
                // notify all subscribers
                for subscriber in self.new_block_subscribers.values() {
                    let _ = subscriber.send(block.clone());
                }
            }
            _ => debug!("new block channel is closed"),
        }
    }
}