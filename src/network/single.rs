use std::sync::{Arc,Mutex};
use std::thread;
use super::{UserBase};
use super::{bound_string};
use super::ClientID;
use ::saving::SaverLoader;

use super::{ProtectedQueue,MsgFromClient,MsgToClientSet,MsgToClient,MsgToServer};


pub fn single_player_login(raw_userbase : &mut UserBase, sl: &SaverLoader) -> ClientID {
    print!("Please give username: ");
    let username = bound_string(super::get_user_string());
    print!("Please give password: ");
    let password = bound_string(super::get_user_string());
    raw_userbase.consume_registration_files(&sl.relative_path(UserBase::REGISTER_PATH));
    raw_userbase.login(username, password)
    .expect("SINGLEPLAYER LOGIN FAIL")
}

pub fn coupler_enter(server_in : Arc<ProtectedQueue<MsgFromClient>>,
                     server_out : Arc<ProtectedQueue<MsgToClientSet>>,
                     client_in : Arc<ProtectedQueue<MsgToClient>>,
                     client_out : Arc<ProtectedQueue<MsgToServer>>,
                     cid : ClientID,
                 ) {
    thread::spawn(move ||{
        //client --> server
        loop {
            let drained : Vec<MsgToServer> = client_out.wait_until_nonempty_drain();
            server_in.lock_pushall_notify(
                drained.into_iter()
                .map(|x| MsgFromClient{msg:x, cid:cid})
            );
        }
    });
    loop {
        let server_outgoing = server_out.wait_until_nonempty_drain();
        let mut actually_send = vec![];
        for s in server_outgoing {
            match s {
                MsgToClientSet::Only(msg, o_cid) => {if cid == o_cid {actually_send.push(msg)}},
                MsgToClientSet::Subset(msg, cid_set) => {if cid_set.get(cid) {actually_send.push(msg)}},
                MsgToClientSet::All(msg) => actually_send.push(msg),
            }
        }
        client_in.lock_pushall_notify(actually_send.into_iter());
    }
}
