pub mod game_state;
mod client_game;
mod server_game;
pub mod entities;
pub mod procedural;
// pub mod server_game_state;

use std::sync::{Arc,Mutex};
use network::messaging::{MsgToClientSet,MsgFromClient,MsgToClient,MsgToServer,Diff};
use network::{ProtectedQueue};
use network::userbase::UserBase;
use super::identity::ClientID;

use super::saving::SaverLoader;


/*
//NOTE consumes caller thread
Manages the shared game state
*/
pub fn server_engine(serv_in : Arc<ProtectedQueue<MsgFromClient>>,
                    serv_out : Arc<ProtectedQueue<MsgToClientSet>>,
                    userbase : Arc<Mutex<UserBase>>,
                    sl : SaverLoader,
                ) {
    server_game::game_loop(serv_in, serv_out, userbase, sl);
}

/*
//NOTE consumes caller thread
Manages the shared game state
*/
pub fn client_engine(client_in : Arc<ProtectedQueue<MsgToClient>>,
                    client_out : Arc<ProtectedQueue<MsgToServer>>,
                    c_id : ClientID) {
    client_game::game_loop(client_in, client_out, c_id);
}
