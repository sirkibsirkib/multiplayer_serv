
use std::path::{PathBuf};

pub enum RunMode {
    ClientPlayer,
    Server,
    SinglePlayer,
}


pub struct Config{
    // maybe_save_game : Option<SavedGame>,
    maybe_save_dir : Option<String>,
    run_mode : RunMode,
    port : Option<u16>,
    host : Option<String>,
}

impl Config {
    pub fn run_mode(&self) -> &RunMode {&self.run_mode}
    pub fn port(&self) -> Option<u16> {self.port}
    pub fn host(&self) -> Option<String> {self.host.clone()}
    pub fn save_dir(&self) -> Option<String> {self.maybe_save_dir.to_owned()}
}

pub fn configure() -> Config {
    let matches = clap_app!(Stat_Keys =>
            (version: "1.0")
            (author: "NAME <email>")
            (about: "decript.")

            (@arg RUN_MODE: +required +takes_value "either `client`, `server` or `single`")
            (@arg IP: -i --ip +takes_value "weefwfe")
            (@arg PORT: -p --port +takes_value "weefwfe")
            (@arg SAVE_PATH: -s --save_path +takes_value "The path to the dir this game's data. Will load from there and save to there.")
        ).get_matches();


    let run_mode = match matches.value_of("RUN_MODE").unwrap() {
        "client" => RunMode::ClientPlayer,
        "server" => RunMode::Server,
        "single" => RunMode::SinglePlayer,
        _ => panic!("NEED TO USE A VALID RUNMODE! SEE --help"),
    };

    Config{
        run_mode : run_mode,
        maybe_save_dir : match matches.value_of("SAVE_PATH") {
            Some(save_dir) => {
                Some(
                    save_dir.to_owned()
                )
            },
            None => None,
        },

        port : match matches.value_of("PORT") {
            Some(s) => Some(s.parse().unwrap()),
            None => None,
        },

        host : match matches.value_of("IP") {
            Some(s) => Some(s.to_owned()),
            None => None,
        },
    }
}
