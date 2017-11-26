use std::sync::{Arc, Mutex, Condvar};
use std::net::TcpListener;
use std::error::Error;
use std::thread;
use std::net::TcpStream;
use std;
use super::saving::SaverLoader;
use std::io::{ErrorKind};
extern crate byteorder;
use std::io::prelude::Read;
use std::io;
use self::byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};
use super::identity::ClientID;

// use std::collections::{HashSet};
use bincode;
use serde::{Serialize,Deserialize};
use std::io::Write;
use std::io::{stdin,stdout};

mod server;
mod client;
pub mod single;
pub mod userbase;
pub mod messaging;

use self::userbase::{UserBase,UserBaseError};

// use super::engine::game_state::{Point};
use self::messaging::*;

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

#[derive(Debug)]
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
        println!("Writing {} bytes message [{}]", bytes.len(), bytes_to_hex(&bytes));
        (&mut num[..]).write_u32::<BigEndian>(bytes.len() as u32)?;
        self.write(&num)?;
        self.write(&bytes)?;
        Ok(())
    }
}

fn bytes_to_hex(bytes : &[u8]) -> String {
    let mut s = String::new();
    for b in bytes {
        s.push_str(&format!("{:X}", b));
    }
    s
}

//everthing is keyed BY CID
//username and password are just client-facing
