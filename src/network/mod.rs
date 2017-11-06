use std::sync::{Arc, Mutex, Condvar};
use std::net::TcpListener;
use std::error::Error;
use std::thread;
use std::net::TcpStream;
use std::fs;
use std;

// use super::bidir_map::BidirMap;
use std::collections::HashMap;

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
                    server_in : Arc<ProtectedQueue<MsgFromClient>>,
                    server_out : Arc<ProtectedQueue<MsgToClientSet>>,
                    userbase : Arc<Mutex<UserBase>>,
                ) -> Result<(), &'static str> {
    if let Ok(listener) = TcpListener::bind(format!("127.0.0.1:{}", port)) {
        thread::spawn(move || {
            server::server_enter(listener, server_in, server_out, userbase);
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
                    client_in : Arc<ProtectedQueue<MsgToClient>>,
                    client_out : Arc<ProtectedQueue<MsgToServer>>,
                ) -> Result<ClientID, &'static Error> {
    //comment
    match TcpStream::connect(format!("{}:{}", host, port)) {
        Ok(mut stream) => {
            //TODO password

            stream.set_read_timeout(None).is_ok();
            let cid = client::client_instigate_handshake(&mut stream);
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

type BoundedString = [u8;32];

pub fn bound_string(s : String) -> BoundedString {
    let mut bounded : BoundedString = [0;32];
    for (i, c) in s.into_bytes().into_iter().take(32).enumerate() {
        bounded[i] = c;
    }
    bounded
}

pub fn bounded_printable(b : BoundedString) -> String {
    let r = std::str::from_utf8(&b).unwrap().trim();
    let q : &str = match r.find(0 as char) {
        Some(ind) => &r[..ind],
        None => r,
    };
    q.trim().to_owned()
}

pub type ClientID = u16;
pub const SINGLE_PLAYER_CID : ClientID = 0;

//PRIMITIVE
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum MsgToServer {
    RequestControlOf(EntityID),
    RelinquishControlof(EntityID),
    CreateEntity(EntityID,Point),
    ControlMoveTo(EntityID,Point),
    //username, password_hash
    ClientLogin(BoundedString,BoundedString),
    LoadEntities, //client needs positions of entities in area
}

//PRIMITIVE
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum MsgToClient {
    CreateEntity(EntityID,Point),
    YouNowControl(EntityID),
    YouNoLongerControl(EntityID),
    EntityMoveTo(EntityID,Point),
    LoginSuccessful(ClientID),
    LoginFailure(UserBaseError),
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
    fn single_read<'a, S>(&mut self, buf : &'a mut [u8]) -> Option<S> where S : Deserialize<'a>;
    fn single_write<S>(&mut self, s : S) where S : Serialize;
    fn single_write_bytes(&mut self, bytes : &[u8]);
}

impl SingleStream for TcpStream {
    fn single_read<'a, S>(&mut self, buf : &'a mut [u8]) -> Option<S> where S : Deserialize<'a> {
        println!("STARTING SINGLE_READ");
        let mut bytes_read : usize = 0;
        while bytes_read < 4 {
            if let Ok(bytes) = self.read(&mut buf[bytes_read..4]){
                if bytes == 0 {
                    return None;
                }
                bytes_read += bytes;
            }
        }
        let num : usize = (&*buf).read_u32::<BigEndian>().unwrap() as usize;
        println!("Received header. will now wait for {} bytes", num);
        let msg_slice = &mut buf[..num];
        self.read_exact(msg_slice).expect("Failed to read exact");
        let stringy = ::std::str::from_utf8(msg_slice).expect("bytes to string");
        println!("got message of len {} : [{:?}]", num, stringy);
        Some(
            serde_json::from_str(&stringy)
            .expect("verify connections to json")
        )
    }

    fn single_write<S>(&mut self, s : S) where S : Serialize {
        println!("STARTING SINGLE_WRITE");
        let stringy = serde_json::to_string(&s).expect("serde outgoing json ONLY");
        let bytes = stringy.as_bytes();
        let mut num : [u8;4] = [0;4];
        println!("Writing {} bytes message `{}`", bytes.len(), &stringy);
        (&mut num[..]).write_u32::<BigEndian>(bytes.len() as u32).is_ok();
        self.write(&num).is_ok();
        self.write(&bytes).is_ok();
    }

    fn single_write_bytes(&mut self, bytes : &[u8]) {
        println!("STARTING single_write_bytes");
        let mut num : [u8;4] = [0;4];
        println!("Writing {} bytes message {:?}", bytes.len(), String::from_utf8_lossy(&bytes));
        (&mut num[..]).write_u32::<BigEndian>(bytes.len() as u32).is_ok();
        self.write(&num).is_ok();
        self.write(&bytes).is_ok();
    }
}

//everthing is keyed BY CID
//username and password are just client-facing

#[derive(Serialize,Deserialize,Debug)]
pub struct UserBase {
    cid_to_username : HashMap<ClientID, BoundedString>,
    username_to_cid : HashMap<BoundedString, ClientID>,
    cid_to_password : HashMap<ClientID, BoundedString>,
    logged_in : HashMap<ClientID, bool>,
    next_avail_cid : ClientID,
}

// use super::saving::SaveLoad;
// impl<'a> SaveLoad<'a> for UserBase{}

impl UserBase {

    pub fn new() -> UserBase {
        UserBase {
            cid_to_username : HashMap::new(),
            username_to_cid : HashMap::new(),
            cid_to_password : HashMap::new(),
            logged_in : HashMap::new(),
            next_avail_cid : 1, //0 reserved for server
        }
    }

    /*
    crawls the given path looking for text files. Registers users and deletes the files when successful
    files are formatted as (inbetween '''):
    '''
    <username>\n
    <password>\n
    '''
    */
    pub fn consume_registration_files(&mut self, path : &str) {
        println!("CONSUMING consume_registration_files");
        let paths = fs::read_dir(path).unwrap();
        for path in paths {
            if let Ok(okpath) = path {
                if let Ok(mut file) = fs::File::open(&okpath.path()) {
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)
                    .expect("something went wrong reading the file");

                    let splits = contents.split("\n").collect::<Vec<&str>>();
                    if splits.len() == 2 {
                        let username : BoundedString = bound_string(splits[0].trim().to_owned());
                        let password : BoundedString = bound_string(splits[1].trim().to_owned());
                        if self.register(username, password) {
                            println!(
                                "Successfully registered {} with pass {}",
                                bounded_printable(username),
                                bounded_printable(password),
                            );
                        } else {
                            println!(
                                "Failed to register {}. User was already registered.",
                                bounded_printable(username),
                            );
                        }
                    }
                }
            }
        }
    }

    //returns true if success
    fn register(&mut self, username : BoundedString, password : BoundedString) -> bool {
        if self.username_to_cid.contains_key(&username) {
            false
        } else {
            let cid = self.next_avail_cid;
            self.next_avail_cid += 1;

            self.username_to_cid.insert(username, cid);
            self.cid_to_username.insert(cid, username);
            self.cid_to_password.insert(cid, password);
            true
        }
    }

    fn is_logged_in(&self, cid : ClientID) -> bool {
        self.logged_in.get(&cid) == Some(&true)
    }

    pub fn login(&mut self, username : BoundedString, password : BoundedString) -> Result<ClientID,UserBaseError> {
        if let Some(cid) = self.username_to_cid.get(&username) {
            if self.is_logged_in(*cid) {
                Err(UserBaseError::AlreadyLoggedIn)
            } else {
                if self.cid_to_password.get(cid) == Some(&password) {
                    self.logged_in.insert(*cid,true);
                    Ok(*cid)
                } else {
                    Err(UserBaseError::WrongPassword)
                }
            }
        } else {
            Err(UserBaseError::UnknownUsername)
        }
    }
}

#[derive(Copy,Clone,Deserialize,Serialize,Debug)]
pub enum UserBaseError {
    AlreadyLoggedIn, UnknownUsername, WrongPassword,
}
