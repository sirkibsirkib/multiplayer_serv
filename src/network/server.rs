use std::sync::{Arc, Mutex};
use std::thread;
use std;
use std::collections::HashMap;
use std::io::prelude::Read;
use std::io::Write;
use std::net::{TcpStream,TcpListener};
use super::{ProtectedQueue,MsgFromClient,MsgToClientSet,ClientID,MsgToServer};
use serde_json;


pub fn server_enter(listener : TcpListener,
                    password : Option<u64>,
                    serv_in : Arc<ProtectedQueue<MsgFromClient>>,
                    serv_out : Arc<ProtectedQueue<MsgToClientSet>>,
                ) {

    //TODO password
    let streams = Arc::new(Mutex::new(HashMap::new()));
    let streams3 = streams.clone();
    println!("Server enter begin");

    thread::spawn(move || {
        listen_for_new_clients(listener, streams, serv_in);
    });
    serve_outgoing(streams3, serv_out);
}

fn listen_for_new_clients(listener : TcpListener,
                          streams : Arc<Mutex<HashMap<ClientID,TcpStream>>>,
                          serv_in : Arc<ProtectedQueue<MsgFromClient>>,
                      ) {
    let mut next_id : ClientID = 0;
    println!("Server listening for clients");
    for s in listener.incoming() {
        let stream = s.unwrap();
        let stream_clone = stream.try_clone().unwrap();
        {
            streams.lock().unwrap().insert(next_id, stream);
        }
        let serv_in_clone = serv_in.clone();
        thread::spawn(move || {
            serve_incoming(next_id, stream_clone, serv_in_clone);
        });
        if next_id == ClientID::max_value() {
            //No more IDs to give! D:
            return
        }
        next_id += 1;
    }
}

fn serve_incoming(c_id : ClientID,
                  mut stream : TcpStream,
                  serv_in : Arc<ProtectedQueue<MsgFromClient>>
              ) {
    let mut buf = [0; 1024];
    loop {
        //blocks until something is there
        match stream.read(&mut buf) {
            Ok(bytes) => {
                let s = std::str::from_utf8(&buf[..bytes]).unwrap();
                let x : MsgToServer = serde_json::from_str(&s).unwrap();
                serv_in.lock_push_notify(MsgFromClient{msg:x, cid:c_id})
            },
            Err(msg) => match msg.kind() {
                std::io::ErrorKind::ConnectionReset => {println!("Connection reset!"); return;},
                x => println!("unexpected kind `{:?}`", x),
            },
        }
    }
}

fn serve_outgoing(streams : Arc<Mutex<HashMap<ClientID,TcpStream>>>,
                  serv_out : Arc<ProtectedQueue<MsgToClientSet>>
              ) {
    let mut msgsets : Vec<MsgToClientSet> = vec![];
    loop {
        //begin loop
        {
            //wait and lock serv_out
            msgsets.extend(serv_out.wait_until_nonempty_drain());
            //unlock serv_out
        }

        {
            //lock streams
            let mut locked_streams = streams.lock().unwrap();
            for m in msgsets.drain(..) {
                match m {
                    MsgToClientSet::Only(msg, c_id) => {
                        if let Some(stream) = locked_streams.get_mut(&c_id){
                            stream.write(serde_json::to_string(&msg).unwrap().as_bytes()).is_ok();
                        }
                    },
                    MsgToClientSet::All(msg) => {
                        let s = serde_json::to_string(&msg).unwrap();
                        for stream in locked_streams.values_mut() {
                            stream.write(s.as_bytes()).is_ok();
                        }
                    },
                }
            }
            //unlock streams
        }
    }
}
