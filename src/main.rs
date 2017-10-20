use std::sync::Arc;
use std::thread;

#[macro_use]
extern crate clap;

mod network;
mod engine;
mod setup;

use network::{ProtectedQueue,MsgToClientSet,MsgFromClient,MsgToClient,MsgToServer};
use setup::RunMode;

fn main() {
    let config = setup::configure();
    let password = config.password();

    match config.run_mode() {
        &RunMode::ClientPlayer => {
            let client_in : Arc<ProtectedQueue<MsgToClient>> = Arc::new(ProtectedQueue::new());
            let client_out : Arc<ProtectedQueue<MsgToServer>> = Arc::new(ProtectedQueue::new());

            let client_in2 = client_in.clone();
            let client_out2 = client_out.clone();

            engine::client_engine(client_in2, client_out2, config.password());
        }

        &RunMode::Server => {
            let serv_out : Arc<ProtectedQueue<MsgToClientSet>> = Arc::new(ProtectedQueue::new());
            let serv_in : Arc<ProtectedQueue<MsgFromClient>> = Arc::new(ProtectedQueue::new());

            let serv_out2 = serv_out.clone();
            let serv_in2 = serv_in.clone();

            engine::server_engine(config.extract_state(), serv_in2, serv_out2, password);
        }

        &RunMode::SinglePlayer => {
            let serv_out : Arc<ProtectedQueue<MsgToClientSet>> = Arc::new(ProtectedQueue::new());
            let serv_in : Arc<ProtectedQueue<MsgFromClient>> = Arc::new(ProtectedQueue::new());
            let client_in : Arc<ProtectedQueue<MsgToClient>> = Arc::new(ProtectedQueue::new());
            let client_out : Arc<ProtectedQueue<MsgToServer>> = Arc::new(ProtectedQueue::new());

            let serv_out2 = serv_out.clone();
            let serv_in2 = serv_in.clone();
            let client_in2 = client_in.clone();
            let client_out2 = client_out.clone();

            network::spawn_coupler(serv_out, serv_in, client_in, client_out);
            thread::spawn(move || {
                engine::client_engine(client_in2, client_out2, password)
            });
            engine::server_engine(config.extract_state(), serv_in2, serv_out2, password);
        }
    }
}
