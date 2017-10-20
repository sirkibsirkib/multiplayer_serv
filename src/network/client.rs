use super::{ProtectedQueue,MsgToClient,MsgToServer};
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

fn client_incoming(mut stream : TcpStream, client_in : Arc<ProtectedQueue<MsgToClient>>) {
    let mut buf = [0; 1024];
    loop {
        //blocks until something is there
        match stream.read(&mut buf) {
            Ok(bytes) => {
                let s = std::str::from_utf8(&buf[..bytes]).unwrap();
                let x : MsgToClient = serde_json::from_str(&s).unwrap();
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
    loop {
        let drained = client_out.wait_until_nonempty_drain();
        for d in drained {
            stream.write(serde_json::to_string(&d).unwrap().as_bytes()).is_ok();
        }
    }
}
