use std::sync::{Arc,Mutex};
use std::thread;

extern crate array_init;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate if_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate clap;

extern crate noise;

extern crate rand;
extern crate bidir_map;
extern crate serde;
extern crate bincode;
extern crate gfx_device_gl;

mod network;
mod engine;
mod setup;
mod saving;
mod identity;
mod utils;
mod points;

use identity::{ClientID};
use network::{ProtectedQueue};
use network::userbase::UserBase;
use network::messaging::{MsgToClientSet,MsgFromClient,MsgToClient,MsgToServer};
use setup::RunMode;
use saving::SaverLoader;

///////////////////////////////////////////////////////////////////////////////////////////////////

fn main() {
    

    let config = setup::configure();

    /*
    See idea.png for an overview.
    The summary is: the players (or player) interact with the "clientside"/local game state / engine directly.
    This engine computes and performs all tasks that it can locally. eg: UI.
    Whenever a change is made that `would` impact the global state in any way, this change is ONLY asynchronously
    requested (dead-reckoning notwithstanding).

    at client-side update() steps, incoming server-sent state changes then actually alter this state.

    This client-server concept means that even singleplayer is composed of a client and server component,
    but they just communicate via shared data rather than over a network. Thus there is very little difference
    between running a client + running a server on one machine vs playing single player (except for some overhead).
    */
    match config.run_mode() {
        &RunMode::ClientPlayer => {

            //these TWO queues represent the two uni-directional channels between client engine and network client.
            let client_in : Arc<ProtectedQueue<MsgToClient>> = Arc::new(ProtectedQueue::new());
            let client_in2 = client_in.clone();
            let client_out : Arc<ProtectedQueue<MsgToServer>> = Arc::new(ProtectedQueue::new());
            let client_out2 = client_out.clone();

            //spawns client in new threads, returns our server-issued client ID. mostly useful for debugging tbh
            let c_id = network::spawn_client(
                &config.host().expect("Need to specify host!"),
                config.port().expect("Need to specify port!"),
                client_in,
                client_out,
            ).expect("Failed to spawn client");
            let sl = SaverLoader::new(&config.save_dir().expect("NO SL DIR"));

            //this call consumes the thread. It begins the client-side game loop
            engine::client_engine(client_in2, client_out2, c_id, sl);
        }

        &RunMode::Server => {

            //these TWO queues represent the two uni-directional channels between server engine and network server.
            let server_in : Arc<ProtectedQueue<MsgFromClient>> = Arc::new(ProtectedQueue::new());
            let server_in2 = server_in.clone();
            let server_out : Arc<ProtectedQueue<MsgToClientSet>> = Arc::new(ProtectedQueue::new());
            let server_out2 = server_out.clone();

            let sl = SaverLoader::new(&config.save_dir().expect("NO SL DIR"));

            let mut raw_userbase = load_user_base(&sl);

            //put file into this directory to register a new user
            raw_userbase.consume_registration_files(&sl.relative_path(UserBase::REGISTER_PATH));

            let userbase : Arc<Mutex<UserBase>> = Arc::new(Mutex::new(raw_userbase));
            let userbase2 : Arc<Mutex<UserBase>> = userbase.clone();

            //spawns a server in new threads.
            network::spawn_server(
                config.port().expect("Need to specify port!"),
                server_in,
                server_out,
                userbase,
                sl.clone(),
            ).expect("FAILED TO SPAWN SERVER");

            let sl = SaverLoader::new(&config.save_dir().expect("NO SL DIR"));

            //consumes this thread to begin the game loop of the global game state aka `server game loop`
            engine::server_engine(server_in2, server_out2, userbase2, sl);
        }

        &RunMode::SinglePlayer => {
            /*
                    --client_out-->         --server_in-->
            CLIENT                  COUPLER                 SERVER
                    <--client_in--         <--server_out--
            */
            let server_in : Arc<ProtectedQueue<MsgFromClient>> = Arc::new(ProtectedQueue::new());
            let server_out : Arc<ProtectedQueue<MsgToClientSet>> = Arc::new(ProtectedQueue::new());
            let client_in : Arc<ProtectedQueue<MsgToClient>> = Arc::new(ProtectedQueue::new());
            let client_out : Arc<ProtectedQueue<MsgToServer>> = Arc::new(ProtectedQueue::new());

            let server_in2 = server_in.clone();
            let server_out2 = server_out.clone();
            let client_in2 = client_in.clone();
            let client_out2 = client_out.clone();


            let sl = SaverLoader::new(&config.save_dir().expect("NO SL DIR"));
            let sl2 = sl.subdir_saver_loader("client_sl_dir/");

            let mut raw_userbase = load_user_base(&sl);

            //spawns a coupler in new threads.
            let cid : ClientID = network::single::single_player_login(&mut raw_userbase, &sl);
            println!("single login {:?}", cid);
            let userbase : Arc<Mutex<UserBase>> = Arc::new(Mutex::new(raw_userbase));
            network::spawn_coupler(server_in, server_out, client_in, client_out, cid);
            thread::spawn(move || {
                engine::server_engine(server_in2, server_out2, userbase, sl);
            });
            //consumes this thread to create client-side aka `local` game loop & engine
            //main thread == client thread. So if piston exists, everything exits
            engine::client_engine(client_in2, client_out2, cid, sl2)
        }
    }
}

fn load_user_base(sl : &SaverLoader) -> UserBase {
    if let Ok(mut loaded) = sl.load_without_key::<UserBase>() {
        println!("loaded userbase file! {:?}", &loaded);
        loaded.log_everyone_out();
        loaded
    } else {
        let u = UserBase::new();
        sl.save_without_key(&u).expect("Save went bad!");
        println!("Created fresh userbase save");
        u
    }
}
