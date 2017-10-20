use std::sync::{Arc, Mutex, Condvar};
use std::net::TcpListener;
use std::error::Error;
use std::thread;
use std::net::TcpStream;

use super::engine::game_state::{EntityID,Point};

mod server;
mod client;
mod single;

/*
NOTE: No need to create a new thread for this call
creates a new server that will listen on the given port, accept clients and forward messages.
The caller can send messages to clients by appending to the MsgToClientSet list.
The caller can receive messages from clients by fetching from the MsgFromClient list.
*/
pub fn spawn_server(port : u16,
                    password : Option<u64>,
                    server_in : Arc<ProtectedQueue<MsgFromClient>>,
                    server_out : Arc<ProtectedQueue<MsgToClientSet>>,
                ) -> Result<(), &'static str> {
    if let Ok(listener) = TcpListener::bind(format!("127.0.0.1:{}", port)) {
        thread::spawn(move || {
            server::server_enter(listener, password, server_in, server_out);
        });
        Ok(())
    } else {
        Err("Couldn't bind!")
    }
}

/*
NOTE: No need to create a new thread for this call
creates a new client that will attempt to connect to a server on the given port
The caller can send messages to the server by appending to the MsgToClientSet list.
The caller can receive messages from the server by fetching from the MsgFromClient list.
*/
pub fn spawn_client(host : &str,
                    port : u16,
                    password : Option<u64>,
                    client_in : Arc<ProtectedQueue<MsgToClient>>,
                    client_out : Arc<ProtectedQueue<MsgToServer>>,
                ) -> Result<ClientID, &'static Error> {
    //comment
    match TcpStream::connect(format!("{}:{}", host, port)) {
        Ok(mut stream) => {
            //TODO password

            stream.set_read_timeout(None).is_ok();
            let cid = client::get_client_id_response(&mut stream);
            thread::spawn(move || {
                client::client_enter(stream, client_in, client_out);
            });
            println!("My CID is {:?}", cid);
            Ok(cid)
        },
        Err(_) => {
            println!("No response.");
            panic!();
        }
    }
}


/*
//NOTE need not start a new thread
For single-player circumstances, the game can instead simply use a coupler
*/
pub fn spawn_coupler(server_in : Arc<ProtectedQueue<MsgFromClient>>,
                     server_out : Arc<ProtectedQueue<MsgToClientSet>>,
                     client_in : Arc<ProtectedQueue<MsgToClient>>,
                     client_out : Arc<ProtectedQueue<MsgToServer>>,
                 ) {
    thread::spawn(move || {
        single::coupler_enter(server_in, server_out, client_in, client_out);
    });
}

///////////////////////////////////////////////////////////////////////////////////////////////////


pub type ClientID = u16;
pub const SINGLE_PLAYER_CID : ClientID = 0;

//PRIMITIVE
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum MsgToServer {
    RequestControlOf(EntityID),
    RelinquishControlof(EntityID),
    CreateEntity(EntityID,Point),
    ControlMoveTo(EntityID,Point),
    ClientIDRequest,
    LoadEntities, //client needs positions of entities in area
}

//PRIMITIVE
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum MsgToClient {
    CreateEntity(EntityID,Point),
    YouNowControl(EntityID),
    YouNoLongerControl(EntityID),
    EntityMoveTo(EntityID,Point),
    ClientIDResponse(ClientID),
}


//WRAPS MsgToServer
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct MsgFromClient {
    pub msg : MsgToServer,
    pub cid : ClientID,
}

//WRAPS MsgToClient
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum MsgToClientSet {
    Only(MsgToClient, ClientID),
    All(MsgToClient),
    // AllExcept(MsgToClient, ClientID),
    // AnyOne(MsgToClient),
    // Specifically(MsgToClient, HashSet<ClientID>),
}

pub struct ProtectedQueue<T> {
    queue : Mutex<Vec<T>>,
    cond : Condvar,
}


impl<T> ProtectedQueue<T> {
    pub fn new() -> Self {
        ProtectedQueue {
            queue : Mutex::new(Vec::new()),
            cond : Condvar::new(),
        }
    }

    pub fn lock_pushall_notify<I>(&self, ts : I)
            where I: Iterator<Item=T>{
        let mut locked_queue = self.queue.lock().unwrap();
        for t in ts {
            locked_queue.push(t);
        }
        self.cond.notify_all();
    }

    pub fn lock_len(&mut self) -> usize {
        let mut locked_queue = self.queue.lock().unwrap();
        locked_queue.len()
    }

    pub fn lock_push_notify(&self, t : T) {
        let mut locked_queue = self.queue.lock().unwrap();
        locked_queue.push(t);
        self.cond.notify_all();
    }

    //locks once. if there is nothing, returns none. if there is something, drains
    pub fn impatient_drain(&self) -> Option<Vec<T>> {
        let mut locked_queue = self.queue.lock().unwrap();
        if locked_queue.is_empty() {
            None
        } else {
            let x : Vec<T> = locked_queue.drain(..).collect();
            Some(x)
        }
    }

    //continuously attempts to lock, sleep and drain until successful
    pub fn wait_until_nonempty_drain(&self) -> Vec<T> {
        let mut locked_queue = self.queue.lock().unwrap();
        while locked_queue.is_empty() {
            locked_queue = self.cond.wait(locked_queue).unwrap();
        }
        let x = locked_queue.drain(..).collect();
        x
    }
}
