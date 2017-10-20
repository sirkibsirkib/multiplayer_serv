use std::sync::{Arc, Mutex, Condvar};
use std::collections::HashSet;
use std::error::Error;
use std::thread;
use std::vec::Drain;

/*
NOTE: No need to create a new thread for this call
creates a new server that will listen on the given port, accept clients and forward messages.
The caller can send messages to clients by appending to the MsgToClientSet list.
The caller can receive messages from clients by fetching from the MsgFromClient list.
*/
fn spawn_server(
    port : u16,
    password : Option<u64>,
    outgoing : Arc<ProtectedQueue<MsgToClientSet>>,
    incoming : Arc<ProtectedQueue<MsgFromClient>>,
) -> Result<(), &'static Error> {
    unimplemented!();
}

/*
NOTE: No need to create a new thread for this call
creates a new client that will attempt to connect to a server on the given port
The caller can send messages to the server by appending to the MsgToClientSet list.
The caller can receive messages from the server by fetching from the MsgFromClient list.
*/
fn spawn_client<I, O>
(
    host : &str,
    port : u16,
    password : Option<u64>,
    outgoing : Arc<ProtectedQueue<MsgToServer>>,
    incoming : Arc<ProtectedQueue<MsgToClient>>
) -> Result<ClientID, &'static Error> {
    unimplemented!();
}


/*
//NOTE need not start a new thread
For single-player circumstances, the game can instead simply use a coupler
*/
pub fn spawn_coupler(
    serv_out : Arc<ProtectedQueue<MsgToClientSet>>,
    serv_in : Arc<ProtectedQueue<MsgFromClient>>,
    client_in : Arc<ProtectedQueue<MsgToClient>>,
    client_out : Arc<ProtectedQueue<MsgToServer>>,
) {
    let only_client_id : ClientID = 0;
    thread::spawn(move ||{
        //client --> server
        loop {
            let drained : Vec<MsgToServer> = client_out.wait_until_nonempty_drain();
            serv_in.lock_pushall_notify(
                drained.into_iter()
                .map(|x| MsgFromClient{msg:x, cid:only_client_id})
            );
        }
    });

    thread::spawn(move ||{
        //server --> client
        let serv_outgoing = serv_out.wait_until_nonempty_drain();
        let mut actually_send = vec![];
        for s in serv_outgoing {
            match s {
                MsgToClientSet::Only(msg, cid) => {if cid == only_client_id {actually_send.push(msg)}},
                MsgToClientSet::All(msg) => actually_send.push(msg),
            }
        }
        client_in.lock_pushall_notify(actually_send.into_iter());
    });
}

///////////////////////////////////////////////////////////////////////////////////////////////////

pub type ClientID = u16;

//PRIMITIVE
pub enum MsgToServer {
    Goodbye, //I am disconnecting
}

//WRAPS MsgToServer
pub struct MsgFromClient {
    msg : MsgToServer,
    cid : ClientID,
}

//PRIMITIVE
pub enum MsgToClient {
    Welcome(ClientID), //you've connected. Here is the ClientID I will use to refer to you
    Shutdown(String), //shut down because of this reason
}

//WRAPS MsgToClient
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

    fn lock_pushall_notify<I>(&self, ts : I)
            where I: Iterator<Item=T>{
        let mut locked_queue = self.queue.lock().unwrap();
        for t in ts {
            locked_queue.push(t);
        }
        self.cond.notify_all();
    }

    fn lock_push_notify(&self, t : T) {
        let mut locked_queue = self.queue.lock().unwrap();
        locked_queue.push(t);
        self.cond.notify_all();
    }

    fn wait_until_nonempty_drain(&self) -> Vec<T> {
        let mut locked_queue = self.queue.lock().unwrap();
        while locked_queue.is_empty() {
            locked_queue = self.cond.wait(locked_queue).unwrap();
        }
        let x = locked_queue.drain(..).collect();
        x
    }
}
