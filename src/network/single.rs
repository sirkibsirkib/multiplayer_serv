use std::sync::{Arc};
use std::thread;

use super::{ProtectedQueue,MsgFromClient,MsgToClientSet,MsgToClient,MsgToServer};
use super::SINGLE_PLAYER_CID;
use super::super::engine::game_state::Point;

pub fn coupler_enter(server_in : Arc<ProtectedQueue<MsgFromClient>>,
                     server_out : Arc<ProtectedQueue<MsgToClientSet>>,
                     client_in : Arc<ProtectedQueue<MsgToClient>>,
                     client_out : Arc<ProtectedQueue<MsgToServer>>,
                 ) {
    thread::spawn(move ||{
        //client --> server
        loop {
            let drained : Vec<MsgToServer> = client_out.wait_until_nonempty_drain();
            server_in.lock_pushall_notify(
                drained.into_iter()
                .map(|x| MsgFromClient{msg:x, cid:SINGLE_PLAYER_CID})
            );
        }
    });
    loop {
        let server_outgoing = server_out.wait_until_nonempty_drain();
        let mut actually_send = vec![];
        for s in server_outgoing {
            match s {
                MsgToClientSet::Only(msg, cid) => {if cid == SINGLE_PLAYER_CID {actually_send.push(msg)}},
                MsgToClientSet::All(msg) => actually_send.push(msg),
            }
        }
        client_in.lock_pushall_notify(actually_send.into_iter());
    }
}
