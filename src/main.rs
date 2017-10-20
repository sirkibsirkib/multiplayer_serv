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

            let c_id = network::spawn_client(
                &config.host().expect("Need to specify host!"),
                config.port().expect("Need to specify port!"),
                config.password(),
                client_in,
                client_out,
            ).expect("Failed to spawn client");
            engine::client_engine(client_in2, client_out2, config.password(), c_id);
        }

        &RunMode::Server => {
            let server_in : Arc<ProtectedQueue<MsgFromClient>> = Arc::new(ProtectedQueue::new());
            let server_out : Arc<ProtectedQueue<MsgToClientSet>> = Arc::new(ProtectedQueue::new());

            let server_out2 = server_out.clone();
            let server_in2 = server_in.clone();


            network::spawn_server(
                config.port().expect("Need to specify port!"),
                config.password(),
                server_in,
                server_out,
            ).expect("FAILED TO SPAWN SERVER");
            engine::server_engine(config.extract_state(), server_in2, server_out2, password);
        }

        &RunMode::SinglePlayer => {
            let server_in : Arc<ProtectedQueue<MsgFromClient>> = Arc::new(ProtectedQueue::new());
            let server_out : Arc<ProtectedQueue<MsgToClientSet>> = Arc::new(ProtectedQueue::new());
            let client_in : Arc<ProtectedQueue<MsgToClient>> = Arc::new(ProtectedQueue::new());
            let client_out : Arc<ProtectedQueue<MsgToServer>> = Arc::new(ProtectedQueue::new());

            let server_in2 = server_in.clone();
            let server_out2 = server_out.clone();
            let client_in2 = client_in.clone();
            let client_out2 = client_out.clone();

            network::spawn_coupler(server_in, server_out, client_in, client_out);
            thread::spawn(move || {
                engine::client_engine(client_in2, client_out2, password, network::SINGPLE_PLAYER_CID)
            });
            engine::server_engine(config.extract_state(), server_in2, server_out2, password);
        }
    }
}
