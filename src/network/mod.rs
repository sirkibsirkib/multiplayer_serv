use std::sync::{Arc, Mutex, Condvar};
use std::net::TcpListener;
use std::error::Error;
use std::thread;
use std::net::TcpStream;
use std::fs;
use std::path::Path;
use std;
use super::saving::SaverLoader;
use std::io::{ErrorKind};
extern crate byteorder;
use std::io::prelude::Read;
use std::io;
use self::byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};
use super::identity::ClientID;

use std::collections::{HashMap,HashSet};
use bincode;
use serde::{Serialize,Deserialize};
use std::io::Write;
use std::io::{stdin,stdout};

mod server;
mod client;
pub mod single;
pub mod messaging;

use super::engine::game_state::{Point};
use self::messaging::{MsgFromClient,MsgToClientSet,MsgToClient,MsgToServer};

pub fn get_user_string() -> String {
    let mut s = String::new();
    let _ = stdout().flush();
    stdin().read_line(&mut s).expect("Did not enter a correct string");
    if let Some('\n')=s.chars().next_back() {
        s.pop();
    }
    if let Some('\r')=s.chars().next_back() {
        s.pop();
    }
    s
}

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
                    sl : SaverLoader,
                ) -> Result<(), &'static str> {
    if let Ok(listener) = TcpListener::bind(format!("127.0.0.1:{}", port)) {
        thread::spawn(move || {
            server::server_enter(listener, server_in, server_out, userbase, sl);
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
                     cid : ClientID,
                 ) {
    thread::spawn(move || {
        single::coupler_enter(server_in, server_out, client_in, client_out, cid);
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


trait SingleStream {
    fn single_read<'a, S>(&mut self, buf : &'a mut [u8]) -> Result<S, io::Error>
        where S : Deserialize<'a>;
    fn single_write<S>(&mut self, s : S) -> Result<(), io::Error>
        where S : Serialize;
    fn single_write_bytes(&mut self, bytes : &[u8]) -> Result<(), io::Error>;
}

impl SingleStream for TcpStream {
    fn single_read<'a, S>(&mut self, buf : &'a mut [u8]) -> Result<S, io::Error>
    where S : Deserialize<'a> {
        println!("STARTING SINGLE_READ");
        let mut bytes_read : usize = 0;
        while bytes_read < 4 {
            let bytes = self.read(&mut buf[bytes_read..4])?;
            if bytes == 0 {
                return Err(io::Error::new(ErrorKind::Other, "zero bytes!"))
            }
            bytes_read += bytes;
        }
        let num : usize = (&*buf).read_u32::<BigEndian>().unwrap() as usize;
        println!("Received header. will now wait for {} bytes", num);
        let msg_slice = &mut buf[..num];
        self.read_exact(msg_slice)?;
        if let Ok(got) = bincode::deserialize(msg_slice) {
            Ok(got)
        } else {
            Err(io::Error::new(ErrorKind::Other, "oh no!"))
        }
    }

    fn single_write<S>(&mut self, s : S) -> Result<(), io::Error>
    where S : Serialize {
        println!("STARTING SINGLE_WRITE");
        // let stringy = serde_json::to_string(&s).expect("serde outgoing json ONLY");
        // let bytes = stringy.as_bytes();
        let bytes = bincode::serialize(&s, bincode::Infinite).expect("went kk lel");
        let mut num : [u8;4] = [0;4];
        // println!("Writing {} bytes message `{}`", bytes.len(), &stringy);
        (&mut num[..]).write_u32::<BigEndian>(bytes.len() as u32)?;
        self.write(&num)?;
        self.write(&bytes)?;
        Ok(())
    }

    fn single_write_bytes(&mut self, bytes : &[u8]) -> Result<(), io::Error> {
        println!("STARTING single_write_bytes");
        let mut num : [u8;4] = [0;4];
        println!("Writing {} bytes message {:?}", bytes.len(), String::from_utf8_lossy(&bytes));
        (&mut num[..]).write_u32::<BigEndian>(bytes.len() as u32)?;
        self.write(&num)?;
        self.write(&bytes)?;
        Ok(())
    }
}

//everthing is keyed BY CID
//username and password are just client-facing

#[derive(Serialize,Deserialize,Debug)]
pub struct UserBase {
    cid_to_username : HashMap<ClientID, BoundedString>,
    username_to_cid : HashMap<BoundedString, ClientID>,
    cid_to_password : HashMap<ClientID, BoundedString>,
    // cid_to_location : HashMap<ClientID, LocationID>,
    // cid_to_controlling : HashMap<ClientID,EntityID>,
    first_time_setup_pending : HashSet<ClientID>,
    logged_in : HashSet<ClientID>,
    next_avail_cid : ClientID,
}

// use super::saving::SaveLoad;
// impl<'a> SaveLoad<'a> for UserBase{}

impl UserBase {

    pub const REGISTER_PATH: &'static str = "users_to_register/";

    pub fn new() -> UserBase {
        UserBase {
            cid_to_username : HashMap::new(),
            username_to_cid : HashMap::new(),
            cid_to_password : HashMap::new(),
            // cid_to_location : HashMap::new(),
            // cid_to_controlling : HashMap::new(),
            first_time_setup_pending : HashSet::new(),
            logged_in : HashSet::new(),
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

    pub fn consume_registration_files(&mut self, path : &Path) {
        println!("CONSUMING consume_registration_files");
        let paths = fs::read_dir(path).expect("Couldn't find relative");
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
                                ":::Successfully registered {} with pass {}",
                                bounded_printable(username),
                                bounded_printable(password),
                            );
                        } else {
                            println!(
                                ":::Failed to register {}. User was already registered.",
                                bounded_printable(username),
                            );
                        }
                    }
                }
                println!("REG NOT REMOVING FILE (debug)", );
                // let _ = fs::remove_file(&okpath.path());
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
            self.first_time_setup_pending.insert(cid);
            true
        }
    }

    fn is_logged_in(&self, cid : ClientID) -> bool {
        self.logged_in.contains(&cid)
    }

    pub fn set_client_setup_true(&mut self, cid : ClientID) {
        self.first_time_setup_pending.remove(&cid);
    }

    pub fn client_is_setup(&self, cid : ClientID) -> bool {
        ! self.first_time_setup_pending.contains(&cid)
    }

    // pub fn set_controlling(&mut self, cid : ClientID, eid : EntityID) {
    //     self.cid_to_controlling.insert(cid, eid);
    // }

    // fn controlling(&self, cid : ClientID) -> Option<EntityID> {
    //     if let Some(eid) = self.cid_to_controlling.get(&cid) {
    //         Some(*eid)
    //     } else {
    //         None
    //     }
    // }
    //
    // pub fn set_location_of(&mut self, cid : ClientID, lid : LocationID) {
    //     self.cid_to_location.insert(cid, lid);
    // }

    // fn location_of(&self, cid : ClientID) -> Option<LocationID> {
    //     if let Some(lid) = self.cid_to_location.get(&cid) {
    //         Some(*lid)
    //     } else {
    //         None
    //     }
    // }

    // USED WHEN LOADED
    //TODO just make serialization omit this field
    pub fn log_everyone_out(&mut self) {
        self.logged_in.clear();
    }

    pub fn logout(&mut self, cid : ClientID) {
        self.logged_in.remove(&cid);
    }

    pub fn login(&mut self, username : BoundedString, password : BoundedString) -> Result<ClientID,UserBaseError> {
        if let Some(cid) = self.username_to_cid.get(&username) {
            if self.is_logged_in(*cid) {
                Err(UserBaseError::AlreadyLoggedIn)
            } else {
                if self.cid_to_password.get(cid) == Some(&password) {
                    self.logged_in.insert(*cid);
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
