pub mod shared_state;

use std::sync::Arc;
use network::{ProtectedQueue,MsgToClientSet,MsgFromClient,MsgToClient,MsgToServer};

use self::shared_state::SharedState;



/*
//NOTE consumes caller thread
Manages the shared game state
*/
pub fn server_engine(initial_state : Option<SharedState>,
                    serv_in : Arc<ProtectedQueue<MsgFromClient>>,
                    serv_out : Arc<ProtectedQueue<MsgToClientSet>>,
                    password : Option<u64>) {

}

/*
//NOTE consumes caller thread
Manages the shared game state
*/
pub fn client_engine(client_in : Arc<ProtectedQueue<MsgToClient>>,
                    client_out : Arc<ProtectedQueue<MsgToServer>>,
                    password : Option<u64>) {

}
