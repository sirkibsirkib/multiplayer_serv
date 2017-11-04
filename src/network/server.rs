use std::sync::{Arc, Mutex};
use std::thread;
use std;
use std::collections::HashMap;
use std::io::prelude::Read;
use std::io::Write;
use std::net::{TcpStream,TcpListener};
use super::{ProtectedQueue,MsgFromClient,MsgToClientSet,ClientID,MsgToServer,MsgToClient,Password};
use serde_json;


pub fn server_enter(listener : TcpListener,
                    password : Password,
                    serv_in : Arc<ProtectedQueue<MsgFromClient>>,
                    serv_out : Arc<ProtectedQueue<MsgToClientSet>>,
                ) {

    //TODO password
    let streams = Arc::new(Mutex::new(HashMap::new()));
    let streams3 = streams.clone();
    println!("Server enter begin. password of {:?}", &password);

    thread::spawn(move || {
        listen_for_new_clients(listener, password, streams, serv_in);
    });
    serve_outgoing(streams3, serv_out);
}

fn listen_for_new_clients(listener : TcpListener,
                          password : Password,
                          streams : Arc<Mutex<HashMap<ClientID,TcpStream>>>,
                          serv_in : Arc<ProtectedQueue<MsgFromClient>>,
                      ) {

    let unverified_connections : Arc<ProtectedQueue<TcpStream>>
        = Arc::new(ProtectedQueue::new());
    let unverified_connections2 = unverified_connections.clone();
    thread::spawn(move || {
        verify_connections(unverified_connections2, password, streams, serv_in);
    });

    println!("Server listening for clients");
    //sleepily handle incoming listeners
    for s in listener.incoming() {
        let stream = s.expect("failed to get incoming stream");
        stream.set_read_timeout(None).is_ok();
        println!("Handing connection to verifier");
        unverified_connections.lock_push_notify(stream);
    }
}

fn verify_connections(unverified : Arc<ProtectedQueue<TcpStream>>,
                      password : Password,
                      streams : Arc<Mutex<HashMap<ClientID,TcpStream>>>,
                      serv_in : Arc<ProtectedQueue<MsgFromClient>>) {
    let mut next_cid : ClientID = 0;
    let mut buf = [0; 1024];
    loop {
        let drained : Vec<TcpStream> = unverified.wait_until_nonempty_drain();
        println!("Verifier thread woke up");
        for mut stream in drained {
            println!("Verifier thread handling a stream");
            loop {
                //keep reading until you see that handshake
                match stream.read(&mut buf) {
                    Ok(bytes) => {
                        let s = std::str::from_utf8(&buf[..bytes]).expect("bytes to string");
                        let x : MsgToServer = serde_json::from_str(&s).expect("verify connections to json");
                        if let MsgToServer::StartHandshake(supplied_password) = x {
                            if supplied_password != password {

                                println!("Refusing client");
                                let refuse = MsgToClient::RefuseHandshake;
                                stream.write(serde_json::to_string(&refuse).expect("refuse to json").as_bytes()).is_ok();
                                break;
                            }
                            println!("Accepting client. assigning CID {}", &next_cid);
                            let accept = MsgToClient::CompleteHandshake(next_cid);
                            stream.write(serde_json::to_string(&accept).expect("accept to json").as_bytes()).is_ok();
                            let stream_clone = stream.try_clone().expect("stream clone");
                            {
                                streams.lock().expect("line 82 lock").insert(next_cid, stream);
                            }
                            let serv_in_clone = serv_in.clone();
                            thread::spawn(move || {
                                serve_incoming(next_cid, stream_clone, serv_in_clone);
                            });
                            if next_cid == ClientID::max_value() {
                                //No more IDs to give! D:
                                return
                            }
                            next_cid += 1;
                            break;
                        }
                    },
                    Err(msg) => match msg.kind() {
                        std::io::ErrorKind::ConnectionReset => {println!("Connection reset!"); return;},
                        x => println!("unexpected kind `{:?}`", x),
                    },
                }
            }
        }
    }
}

fn serve_incoming(c_id : ClientID,
                  mut stream : TcpStream,
                  serv_in : Arc<ProtectedQueue<MsgFromClient>>
              ) {
    println!("Dedicated thread for incoming from cid {}", c_id);
    let mut buf = [0; 1024];
    loop {
        //blocks until something is there
        match stream.read(&mut buf) {
            Ok(bytes) => {
                let s = std::str::from_utf8(&buf[..bytes]).expect("serve incoming bytes to str");
                let x : MsgToServer = serde_json::from_str(&s).expect("serde json serve incoming");
                println!("server incoming read of {:?} from {:?}", &x, &c_id);
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
    println!("Serving outgoing updates");
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
            let mut locked_streams = streams.lock().expect("lock streams serve outgoing");
            for m in msgsets.drain(..) {
                match m {
                    MsgToClientSet::Only(msg, c_id) => {
                        if let Some(stream) = locked_streams.get_mut(&c_id){
                            println!("server outgoing write of {:?} to {:?}", &msg, &c_id);
                            stream.write(serde_json::to_string(&msg).expect("serde outgoing json ONLY").as_bytes()).is_ok();
                        }
                    },
                    MsgToClientSet::All(msg) => {
                        println!("server outgoing write of {:?} to ALL", &msg);
                        let s = serde_json::to_string(&msg).expect("serde outgoing json ALL");
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
