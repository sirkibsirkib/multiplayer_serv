use super::engine::shared_state::SharedState;
use std::error::Error;


pub enum RunMode {
    ClientPlayer,
    Server,
    SinglePlayer,
}


pub struct Config{
    maybe_state : Option<SharedState>,
    run_mode : RunMode,
    password : Option<u64>,
}

impl Config {
    pub fn maybe_state(&self) -> &Option<SharedState> {&self.maybe_state}
    pub fn run_mode(&self) -> &RunMode {&self.run_mode}
    pub fn password(&self) -> Option<u64> {self.password}
    pub fn extract_state(self) -> Option<SharedState> {self.maybe_state}
}

pub fn configure() -> Config {
    let matches = clap_app!(Stat_Keys =>
            (version: "1.0")
            (author: "NAME <email>")
            (about: "decript.")

            (@arg RUN_MODE: +required +takes_value "TODO")
            (@arg LOAD_PATH: -l --load_path +takes_value "TODO")
            (@arg PASSWORD: -p --password +takes_value "TODO")
        ).get_matches();


    let run_mode = match matches.value_of("RUN_MODE").unwrap() {
        "client" => RunMode::ClientPlayer,
        "server" => RunMode::Server,
        "single" => RunMode::SinglePlayer,
        _ => panic!("NEED TO USE A VALID RUNMODE! SEE --help"),
    };

    if let RunMode::ClientPlayer = run_mode {
        if matches.is_present("LOAD_PATH"){
            panic!("Client cant load from a path!");
        }
    }

    if let RunMode::SinglePlayer = run_mode {
        if matches.is_present("Single player doesn't use a password, as there is no server!"){
            panic!();
        }
    }

    Config{
        run_mode : run_mode,
        maybe_state : match matches.value_of("load_path") {
            Some(s) => Some(load_from(s).unwrap()),
            None => None,
        },
        password : match matches.value_of("PASSWORD") {
            Some(s) => Some(s.parse().unwrap()),
            None => None,
        },
    }
}

fn load_from(path : &str) -> Result<SharedState, &'static Error> {
    Ok(SharedState::new())
}
