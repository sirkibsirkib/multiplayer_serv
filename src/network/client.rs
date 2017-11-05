use super::{ProtectedQueue,MsgToClient,MsgToServer,ClientID,BoundedString};
use std::sync::Arc;
use std::io::Write;
use std::io::prelude::Read;
use std::net::TcpStream;
use std::thread;
use serde_json;
use std::time;
use std::io::{stdin,stdout};
use super::bound_string;
use std;

use super::SingleStream;

pub fn client_enter(stream : TcpStream,
                    client_in : Arc<ProtectedQueue<MsgToClient>>,
                    client_out : Arc<ProtectedQueue<MsgToServer>>,
                ){
    //comment
    let stream_clone = stream.try_clone().expect("client stream clone");
    thread::spawn(move || {
        client_incoming(stream, client_in);
    });
    client_outgoing(stream_clone, client_out);
}

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

pub fn client_instigate_handshake(stream : &mut TcpStream) -> ClientID {
    let mut buf = [0; 1024];
    let short_timeout = time::Duration::from_millis(100);
    stream.set_read_timeout(Some(short_timeout)).is_ok();

    print!("Please give username: ");
    let username = bound_string(get_user_string());
    print!("Please give password: ");
    let password = bound_string(get_user_string());

    //pre-build password bytes
    let password_msg = serde_json::to_string(&MsgToServer::ClientLogin(username, password)).expect("handshake to str");
    let password_bytes = password_msg.as_bytes();
    stream.single_write_bytes(password_bytes);

    loop {
        if let Some(msg) = stream.single_read(&mut buf){
            if let MsgToClient::LoginSuccessful(cid) = msg {
                stream.set_read_timeout(None).is_ok();
                return cid
            } else if let MsgToClient::BadUsername = msg {
                panic!("Bad username!!");

            } else if let MsgToClient::BadPassword = msg {
                panic!("Bad password!!");

            } else {
                //unexpected message. Ignore.
            }
        } else {
            // timeout. resending
            //TODO it should never timeout as packets won't get lost
            stream.single_write_bytes(password_bytes);
        }
    }
}

fn client_incoming(mut stream : TcpStream, client_in : Arc<ProtectedQueue<MsgToClient>>) {
    println!("Listening for incoming messages");
    let mut buf = [0; 1024];
    loop {
        //blocks until something is there
        let msg : MsgToClient = stream.single_read(&mut buf).unwrap();
        //TODO catch connection reset etc.
        println!("client incoming read of {:?}", &msg);
        client_in.lock_push_notify(msg);
    }
}

fn client_outgoing(mut stream : TcpStream, client_out : Arc<ProtectedQueue<MsgToServer>>) {
    println!("Listening for outgoing messages");
    loop {
        let drained = client_out.wait_until_nonempty_drain();
        for d in drained {
            println!("client outgoing write of {:?}", &d);
            stream.single_write(d);
        }
    }
}
