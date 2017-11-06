use std::sync::{Arc, Mutex};
use std::thread;
use std;
use std::collections::HashMap;
use std::io::prelude::Read;
use std::net::{TcpStream,TcpListener};
use super::{ProtectedQueue,MsgFromClient,MsgToClientSet,ClientID,MsgToServer,MsgToClient,BoundedString,UserBase,UserBaseError};
use serde_json;
use super::SingleStream;
use super::bound_string;
use std::fs::File;



pub fn server_enter(listener : TcpListener,
                    serv_in : Arc<ProtectedQueue<MsgFromClient>>,
                    serv_out : Arc<ProtectedQueue<MsgToClientSet>>,
                    userbase : Arc<Mutex<UserBase>>,
                ) {

    //TODO password
    let streams = Arc::new(Mutex::new(HashMap::new()));
    let streams3 = streams.clone();
    println!("Server enter begin.");

    thread::spawn(move || {
        listen_for_new_clients(listener, streams, serv_in, userbase);
    });
    serve_outgoing(streams3, serv_out);
}

fn listen_for_new_clients(listener : TcpListener,
                          streams : Arc<Mutex<HashMap<ClientID,TcpStream>>>,
                          serv_in : Arc<ProtectedQueue<MsgFromClient>>,
                          userbase : Arc<Mutex<UserBase>>,
                      ) {

    let unverified_connections : Arc<ProtectedQueue<TcpStream>>
        = Arc::new(ProtectedQueue::new());
    let unverified_connections2 = unverified_connections.clone();
    thread::spawn(move || {
        verify_connections(unverified_connections2, streams, serv_in, userbase);
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
//
// fn get_password_for_username(b : BoundedString) -> Option<BoundedString> {
//     let usrname_path = [
//         "./accounts/user_credentials/",
//         std::str::from_utf8(&b).unwrap().trim_matches(|x| x as u8 == 0),
//         ".txt"
//     ].join("");
//     println!("Looking for file in {}", &usrname_path);
//     if let Ok(mut f) = File::open(usrname_path) {
//         let mut contents = String::new();
//         f.read_to_string(&mut contents)
//             .expect("something went wrong reading the file");
//         Some(bound_string(contents))
//     } else {
//         //couldn't find this user's file
//         None
//     }
// }

fn verify_connections(unverified : Arc<ProtectedQueue<TcpStream>>,
                      streams : Arc<Mutex<HashMap<ClientID,TcpStream>>>,
                      serv_in : Arc<ProtectedQueue<MsgFromClient>>,
                      userbase : Arc<Mutex<UserBase>>,
                  ) {
    let mut buf = [0; 1024];
    loop {
        let drained : Vec<TcpStream> = unverified.wait_until_nonempty_drain();
        println!("Verifier thread woke up");
        for mut stream in drained {
            println!("Verifier thread handling a stream");
            //TODO instead of unwrap, use something else
            let msg : MsgToServer = stream.single_read(&mut buf).unwrap();

            userbase.lock().unwrap().consume_registration_files("./registration_files/");

            if let MsgToServer::ClientLogin(username, password) = msg {
                match userbase.lock().unwrap().login(username, password) {
                    Err(ub_error) => {
                        stream.single_write(MsgToClient::LoginFailure(ub_error));
                    },
                    Ok(cid) => {
                        stream.single_write(MsgToClient::LoginSuccessful(cid));
                        let stream_clone = stream.try_clone().expect("stream clone");
                        {
                            streams.lock().expect("line 82 lock").insert(cid, stream);
                        }
                        let serv_in_clone = serv_in.clone();
                        thread::spawn(move || {
                            serve_incoming(cid, stream_clone, serv_in_clone);
                        });
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
        let msg : MsgToServer = stream.single_read(&mut buf)
                                .expect("couldn't unwrap single server:107");
        println!("server incoming read of {:?} from {:?}", &msg, &c_id);
        serv_in.lock_push_notify(MsgFromClient{msg:msg, cid:c_id})
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
                            stream.single_write(msg);
                        }
                    },
                    MsgToClientSet::All(msg) => {
                        println!("server outgoing write of {:?} to ALL", &msg);
                        let temp = serde_json::to_string(&msg)
                                    .expect("serde outgoing json ALL");
                        let msg_bytes : &[u8] = temp.as_bytes();
                        for stream in locked_streams.values_mut() {
                            stream.single_write_bytes(msg_bytes);
                        }
                    },
                }
            }
            //unlock streams
        }
    }
}
