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
Creates autonomous server that will attempt to drain server_in and populate server_out.
Runs in several threads. has popup threads for incoming clients. accepts new clients and issues then CIDs

               --server_in-->          ~~TCP packets~~>  ||
SERVER_ENGINE                  SERVER                    ||network
               <--server_out--         <~~TCP packets~~  ||

NOTE: Does NOT consume caller thread
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
Creates autonomous server that will attempt to drain server_in and populate server_out.
Runs in several threads. has popup threads for incoming clients. accepts new clients and issues then CIDs

                --client_in-->          ~~TCP packets~~>  ||
CLIENT_ENGINE                   CLIENT                    ||network
                <--client_out--         <~~TCP packets~~  ||

NOTE: Does NOT consume caller thread
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
            let cid = client::client_instigate_handshake(&mut stream, password);
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
couples server_in with client_out, client_in with server_out.
Greedily drains OUTs and populates INs.
Outside engines can use this coupler by draining their respective IN and populating their OUT
NOTE: Does NOT consume caller thread
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
pub type Password = Option<u64>;
pub const SINGLE_PLAYER_CID : ClientID = 0;

//PRIMITIVE
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum MsgToServer {
    RequestControlOf(EntityID),
    RelinquishControlof(EntityID),
    CreateEntity(EntityID,Point),
    ControlMoveTo(EntityID,Point),
    StartHandshake(Password),
    LoadEntities, //client needs positions of entities in area
}

//PRIMITIVE
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum MsgToClient {
    CreateEntity(EntityID,Point),
    YouNowControl(EntityID),
    YouNoLongerControl(EntityID),
    EntityMoveTo(EntityID,Point),
    CompleteHandshake(ClientID),
    RefuseHandshake,
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
        let locked_queue = self.queue.lock().unwrap();
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

use serde_json;

use serde::de::Deserialize;
use serde::ser::Serialize;
extern crate byteorder;
use std::io::Write;
use std::io::prelude::Read;

use self::byteorder::{ReadBytesExt, WriteBytesExt, BigEndian, LittleEndian};

trait SingleStream {
    fn single_read<'a, S>(&mut self, buf : &'a mut [u8]) -> S where S : Deserialize<'a>;
    fn single_write<S>(&mut self, s : S) where S : Serialize;
}

impl SingleStream for TcpStream {
    fn single_read<'a, S>(&mut self, buf : &'a mut [u8]) -> S where S : Deserialize<'a> {
        let mut bytes_read : usize = 0;
        while bytes_read < 4 {
            if let Ok(bytes) = self.read(&mut buf[bytes_read..]){
                bytes_read += bytes;
            }
        }
        let num : usize = (&*buf).read_u32::<BigEndian>().unwrap() as usize;
        let msg_slice = &mut buf[4..4+num];
        self.read_exact(msg_slice).expect("Failed to read exact");
        let stringy = ::std::str::from_utf8(msg_slice).expect("bytes to string");
        println!("got message of len {} : [{:?}]", num, stringy);
        serde_json::from_str(&stringy).expect("verify connections to json")

    }

    fn single_write<S>(&mut self, s : S) where S : Serialize{
        let stringy = serde_json::to_string(&s).expect("serde outgoing json ONLY");
        let bytes = stringy.as_bytes();
        let mut num : [u8;4] = [0;4];
        println!("Writing {} bytes message `{}`", bytes.len(), &stringy);
        (&mut num[..]).write_u32::<BigEndian>(bytes.len() as u32).is_ok();
        self.write(&num).is_ok();
        self.write(&bytes).is_ok();
    }
}
