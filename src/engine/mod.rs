pub mod game_state;
mod client_game;
mod server_game;
mod locations;

use std::sync::{Arc,Mutex};
use network::{ProtectedQueue,MsgToClientSet,MsgFromClient,MsgToClient,MsgToServer,ClientID,UserBase};
use std::path::PathBuf;

use self::game_state::GameState;
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
