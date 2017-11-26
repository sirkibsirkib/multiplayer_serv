
// use serde::{Serialize,Deserialize};
use std::collections::{HashMap,HashSet};
use ::identity::{ClientID};
use super::{BoundedString,bound_string,bounded_printable};
use std::fs;
use std::io::prelude::Read;
// use std::io;
use std::path::Path;
use utils::traits::*;

#[derive(Serialize,Deserialize,Debug)]
pub struct UserBase {
    cid_to_username : HashMap<ClientID, BoundedString>,
    username_to_cid : HashMap<BoundedString, ClientID>,
    cid_to_password : HashMap<ClientID, BoundedString>,
    first_time_setup_pending : HashSet<ClientID>,
    logged_in : HashSet<ClientID>,
    next_avail_cid : ClientID,
}

impl KnowsSavePrefix for UserBase {
    fn get_save_prefix() -> String {
        "userbase".to_owned()
    }
}

impl UserBase {
    pub const SAVE_PATH: &'static str = "user_base.lel";
    pub const REGISTER_PATH: &'static str = "users_to_register/";

    pub fn new() -> UserBase {
        UserBase {
            cid_to_username : HashMap::new(),
            username_to_cid : HashMap::new(),
            cid_to_password : HashMap::new(),
            // cid_to_location : HashMap::new(),
            // cid_to_controlling : HashMap::new(),
            first_time_setup_pending : HashSet::new(),
            logged_in : HashSet::new(),
            next_avail_cid : 1, //0 reserved for server
        }
    }

    /*
    crawls the given path looking for text files. Registers users and deletes the files when successful
    files are formatted as (inbetween '''):
    '''
    <username>\n
    <password>\n
    '''
    */
    pub fn consume_registration_files(&mut self, path : &Path) {
        println!("CONSUMING consume_registration_files");
        let paths = fs::read_dir(path).expect("Couldn't find relative");
        for path in paths {
            if let Ok(okpath) = path {
                if let Ok(mut file) = fs::File::open(&okpath.path()) {
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)
                    .expect("something went wrong reading the file");

                    let splits = contents.split("\n").collect::<Vec<&str>>();
                    if splits.len() == 2 {
                        let username : BoundedString = bound_string(splits[0].trim().to_owned());
                        let password : BoundedString = bound_string(splits[1].trim().to_owned());
                        if self.register(username, password) {
                            println!(
                                ":::Successfully registered {} with pass {}",
                                bounded_printable(username),
                                bounded_printable(password),
                            );
                        } else {
                            println!(
                                ":::Failed to register {}. User was already registered.",
                                bounded_printable(username),
                            );
                        }
                    }
                }
                println!("REG NOT REMOVING FILE (debug)", );
                // let _ = fs::remove_file(&okpath.path());
            }
        }
    }

    //returns true if success
    fn register(&mut self, username : BoundedString, password : BoundedString) -> bool {
        if self.username_to_cid.contains_key(&username) {
            false
        } else {
            let cid = self.next_avail_cid;
            self.next_avail_cid += 1;

            self.username_to_cid.insert(username, cid);
            self.cid_to_username.insert(cid, username);
            self.cid_to_password.insert(cid, password);
            self.first_time_setup_pending.insert(cid);
            true
        }
    }

    fn is_logged_in(&self, cid : ClientID) -> bool {
        self.logged_in.contains(&cid)
    }

    pub fn set_client_setup_true(&mut self, cid : ClientID) {
        self.first_time_setup_pending.remove(&cid);
    }

    pub fn client_is_setup(&self, cid : ClientID) -> bool {
        ! self.first_time_setup_pending.contains(&cid)
    }

    // USED WHEN LOADED
    //TODO just make serialization omit this field
    pub fn log_everyone_out(&mut self) {
        self.logged_in.clear();
    }

    pub fn logout(&mut self, cid : ClientID) {
        println!("Userbase LOGGING OUT{:?}", cid);
        self.logged_in.remove(&cid);
        println!("Userbase state : {:?}", self);
    }

    pub fn login(&mut self, username : BoundedString, password : BoundedString) -> Result<ClientID,UserBaseError> {
        println!("Login attempt from <{:?}> <{:?}>", &bounded_printable(username), &bounded_printable(password));
        if let Some(cid) = self.username_to_cid.get(&username) {
            if self.is_logged_in(*cid) {
                Err(UserBaseError::AlreadyLoggedIn)
            } else {
                if self.cid_to_password.get(cid) == Some(&password) {
                    self.logged_in.insert(*cid);
                    Ok(*cid)
                } else {
                    Err(UserBaseError::WrongPassword)
                }
            }
        } else {
            Err(UserBaseError::UnknownUsername)
        }
    }
}

#[derive(Copy,Clone,Deserialize,Serialize,Debug)]
pub enum UserBaseError {
    AlreadyLoggedIn, UnknownUsername, WrongPassword,
}
