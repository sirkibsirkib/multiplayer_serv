use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;
use std::net::{TcpStream,TcpListener};
use super::{ProtectedQueue,MsgFromClient,MsgToClientSet,ClientID,MsgToServer,MsgToClient,UserBase};
use bincode;
use super::SingleStream;
use super::super::saving::SaverLoader;
use std::time;



pub fn server_enter(listener : TcpListener,
                    serv_in : Arc<ProtectedQueue<MsgFromClient>>,
                    serv_out : Arc<ProtectedQueue<MsgToClientSet>>,
                    userbase : Arc<Mutex<UserBase>>,
                    sl : SaverLoader,
                ) {

    //TODO password
    let streams = Arc::new(Mutex::new(HashMap::new()));
    let streams3 = streams.clone();
    println!("Server enter begin.");

    let userbase_clone = userbase.clone();
    thread::spawn(move || {
        listen_for_new_clients(listener, streams, serv_in, userbase_clone, sl);
    });
    serve_outgoing(streams3, serv_out, userbase);
}

fn listen_for_new_clients(listener : TcpListener,
                          streams : Arc<Mutex<HashMap<ClientID,TcpStream>>>,
                          serv_in : Arc<ProtectedQueue<MsgFromClient>>,
                          userbase : Arc<Mutex<UserBase>>,
                          sl : SaverLoader,
                      ) {

    let unverified_connections : Arc<ProtectedQueue<TcpStream>>
        = Arc::new(ProtectedQueue::new());
    let unverified_connections2 = unverified_connections.clone();
    thread::spawn(move || {
        verify_connections(unverified_connections2, streams, serv_in, userbase, sl);
    });

    println!("Server listening for clients");
    //sleepily handle incoming listeners
    for s in listener.incoming() {
        let stream = s.expect("failed to get incoming stream");
        println!("Handing connection to verifier");
        unverified_connections.lock_push_notify(stream);
    }
}


fn verify_connections(unverified : Arc<ProtectedQueue<TcpStream>>,
                      streams : Arc<Mutex<HashMap<ClientID,TcpStream>>>,
                      serv_in : Arc<ProtectedQueue<MsgFromClient>>,
                      userbase : Arc<Mutex<UserBase>>,
                      sl : SaverLoader,
                  ) {
    let mut buf = [0; 1024];
    let verify_timeout = time::Duration::from_millis(20000);
    loop {
        let drained : Vec<TcpStream> = unverified.wait_until_nonempty_drain();
        println!("Verifier thread woke up");
        for mut stream in drained {

            //TODO spawn thread to wait for username message here! so don't have to wait

            println!(":::Verifier thread handling a stream");
            stream.set_read_timeout(Some(verify_timeout)).is_ok();
            //TODO instead of unwrap, use something else
            if let Ok(msg) = stream.single_read(&mut buf) {
                println!(":::got message to verify ");
                userbase.lock().unwrap().consume_registration_files(&sl.relative_path(UserBase::REGISTER_PATH));
                if let MsgToServer::ClientLogin(username, password) = msg {
                    match userbase.lock().unwrap().login(username, password) {
                        Err(ub_error) => {
                            //don't care if it fails
                            let _ = stream.single_write(MsgToClient::LoginFailure(ub_error));
                            drop(stream); //not necessary, just for clarity
                        },
                        Ok(cid) => {
                            if let Err(_) = stream.single_write(MsgToClient::LoginSuccessful(cid)) {
                                println!("O shet");
                            } else {
                                stream.set_read_timeout(None).is_ok();
                                let stream_clone = stream.try_clone().expect("stream clone");
                                {
                                    streams.lock().expect("line 82 lock").insert(cid, stream);
                                }
                                let serv_in_clone = serv_in.clone();
                                thread::spawn(move || {
                                    serve_incoming(cid, stream_clone, serv_in_clone);
                                });
                            }
                        },
                    }
                }
            } else {
                //close the connection
                println!("Client timeout!");
                drop(stream);
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
        if let Ok(msg) = stream.single_read(&mut buf) {
            println!("server incoming read of {:?} from {:?}", &msg, &c_id);
            serv_in.lock_push_notify(MsgFromClient{msg:msg, cid:c_id})
        } else {
            println!("INCOMING SERVER DROPPING");
            drop(stream);
            break;
        }
    }
}

fn serve_outgoing(streams : Arc<Mutex<HashMap<ClientID,TcpStream>>>,
                  serv_out : Arc<ProtectedQueue<MsgToClientSet>>,
                  userbase : Arc<Mutex<UserBase>>,
              ) {
    println!("Serving outgoing updates");
    let mut streams_to_remove : Vec<ClientID> = vec![];
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
            println!("have {:?} active clients", locked_streams.len());
            if !streams_to_remove.is_empty() {
                for cid in streams_to_remove.drain(..) {
                    println!("output is pruning stream {}", cid);
                    locked_streams.remove(&cid);
                    let mut locked_userbase = userbase.lock().unwrap();
                    locked_userbase.logout(cid);
                }
            }
            for m in msgsets.drain(..) {
                match m {
                    MsgToClientSet::Only(msg, cid) => {
                        if let Some(stream) = locked_streams.get_mut(&cid){
                            println!("server outgoing write of {:?} to {:?}", &msg, &cid);
                            if let Err(_) = stream.single_write(msg) {
                                streams_to_remove.push(cid);
                            }
                        }
                    },
                    MsgToClientSet::All(msg) => {
                        println!("server outgoing write of {:?} to ALL", &msg);
                        let msg_bytes = bincode::serialize(&msg, bincode::Infinite).expect("ech");
                        for (cid, stream) in locked_streams.iter_mut() {
                            if let Err(_) = stream.single_write_bytes(&msg_bytes) {
                                streams_to_remove.push(*cid);
                            }
                        }
                    },
                }
            }
            //unlock streams
        }
    }
}
