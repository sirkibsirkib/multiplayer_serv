pub mod game_state;
mod client_game;
mod server_game;
mod locations;

use std::sync::Arc;
use network::{ProtectedQueue,MsgToClientSet,MsgFromClient,MsgToClient,MsgToServer,ClientID};

use self::game_state::GameState;



/*
//NOTE consumes caller thread
Manages the shared game state
*/
pub fn server_engine(initial_state : Option<GameState>,
                    serv_in : Arc<ProtectedQueue<MsgFromClient>>,
                    serv_out : Arc<ProtectedQueue<MsgToClientSet>>) {
    let global_state = if let Some(s) = initial_state {
        s
    } else {
        GameState::new()
    };
    server_game::game_loop(global_state, serv_in, serv_out);
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
