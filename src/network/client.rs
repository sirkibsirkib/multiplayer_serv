use super::{ProtectedQueue,MsgToClient,MsgToServer,ClientID};
use std::sync::Arc;
use std::net::TcpStream;
use std::thread;
use bincode;
use std::time;
use super::bound_string;
use super::UserBaseError;

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

pub fn client_instigate_handshake(stream : &mut TcpStream) -> ClientID {
    let mut buf = [0; 1024];
    let short_timeout = time::Duration::from_millis(100);
    stream.set_read_timeout(Some(short_timeout)).is_ok();

    print!("Please give username: ");
    let username = bound_string(super::get_user_string());
    print!("Please give password: ");
    let password = bound_string(super::get_user_string());

    //pre-build password bytes
    // let password_msg = serde_json::to_string(&MsgToServer::ClientLogin(username, password)).expect("handshake to str");
    let password_bytes = bincode::serialize(&MsgToServer::ClientLogin(username, password), bincode::Infinite).expect("ser handshake");
    // let password_bytes = password_msg.as_bytes();
    if let Err(_) = stream.single_write_bytes(&password_bytes) {
        println!("WTF");
        drop(stream);
        panic!("WHEEE");
    }

    loop {
        if let Ok(msg) = stream.single_read(&mut buf) {
            if let MsgToClient::LoginSuccessful(cid) = msg {
                stream.set_read_timeout(None).is_ok();
                return cid
            } else if let MsgToClient::LoginFailure(ub_error) = msg {
                match ub_error {
                    UserBaseError::AlreadyLoggedIn => panic!("You are already logged in!"),
                    UserBaseError::UnknownUsername => panic!("Unknown username! Register first"),
                    UserBaseError::WrongPassword => panic!("Password doesn't match"),
                }

            } else {
                //unexpected message. Ignore.
            }
        } else {
            // timeout. resending
            if let Err(_) = stream.single_write_bytes(&password_bytes) {
                println!("client handshake bad");
                drop(stream);
                panic!("AAAH");
            }
        }
    }
}

fn client_incoming(mut stream : TcpStream, client_in : Arc<ProtectedQueue<MsgToClient>>) {
    println!("Listening for incoming messages");
    let mut buf = [0; 1024];
    loop {
        //blocks until something is there
        if let Ok(msg) = stream.single_read(&mut buf) {
            println!("client incoming read of {:?}", &msg);
            client_in.lock_push_notify(msg);
        } else {
            println!("Client dropping incoming");
            drop(stream);
            break;
        }
    }
}

fn client_outgoing(mut stream : TcpStream, client_out : Arc<ProtectedQueue<MsgToServer>>) {
    println!("Listening for outgoing messages");
    loop {
        let drained = client_out.wait_until_nonempty_drain();
        for d in drained {
            println!("client outgoing write of {:?}", &d);
            if let Err(_) = stream.single_write(d) {
                println!("client out dropping");
                drop(stream);
                return;
            }
        }
    }
}
