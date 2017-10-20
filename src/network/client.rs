use super::{ProtectedQueue,MsgToClient,MsgToServer,ClientID};
use std::sync::Arc;
use std::io::Write;
use std::io::prelude::Read;
use std::net::TcpStream;
use std::thread;
use serde_json;
use std;

pub fn client_enter(stream : TcpStream,
                    client_in : Arc<ProtectedQueue<MsgToClient>>,
                    client_out : Arc<ProtectedQueue<MsgToServer>>,
                ){
    //comment
    let stream_clone = stream.try_clone().unwrap();
    thread::spawn(move || {
        client_incoming(stream, client_in);
    });
    client_outgoing(stream_clone, client_out);
}

pub fn get_client_id_response(stream : &mut TcpStream) -> ClientID {
    let mut buf = [0; 1024];
    stream.write(serde_json::to_string(&MsgToServer::ClientIDRequest).unwrap().as_bytes()).is_ok();
    loop {
        if let Ok(bytes) = stream.read(&mut buf) {
            let s = std::str::from_utf8(&buf[..bytes]).unwrap();
            let x : MsgToClient = serde_json::from_str(&s).unwrap();
            if let MsgToClient::ClientIDResponse(cid)  = x {
                return cid
            }
            //TODO re-send
        } else {
            panic!();
        }
    }
}

fn client_incoming(mut stream : TcpStream, client_in : Arc<ProtectedQueue<MsgToClient>>) {
    println!("Listening for incoming messages");
    let mut buf = [0; 1024];
    loop {
        //blocks until something is there
        match stream.read(&mut buf) {
            Ok(bytes) => {
                let s = std::str::from_utf8(&buf[..bytes]).unwrap();
                let x : MsgToClient = serde_json::from_str(&s).unwrap();
                println!("client incoming read of {:?}", &x);
                client_in.lock_push_notify(x);
            },
            Err(msg) => match msg.kind() {
                std::io::ErrorKind::ConnectionReset => {println!("Connection reset!"); return;},
                x => println!("unexpected kind `{:?}`", x),
            },
        }
    }
}

fn client_outgoing(mut stream : TcpStream, client_out : Arc<ProtectedQueue<MsgToServer>>) {
    println!("Listening for outgoing messages");
    loop {
        let drained = client_out.wait_until_nonempty_drain();
        for d in drained {
            println!("client outgoing write of {:?}", &d);
            stream.write(serde_json::to_string(&d).unwrap().as_bytes()).is_ok();
        }
    }
}
