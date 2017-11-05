use super::engine::game_state::GameState;

pub enum RunMode {
    ClientPlayer,
    Server,
    SinglePlayer,
}

pub struct Config{
    maybe_state : Option<GameState>,
    run_mode : RunMode,
    port : Option<u16>,
    host : Option<String>,
}

impl Config {
    pub fn run_mode(&self) -> &RunMode {&self.run_mode}
    pub fn port(&self) -> Option<u16> {self.port}
    pub fn host(&self) -> Option<String> {self.host.clone()}
    pub fn extract_state(self) -> Option<GameState> {self.maybe_state}
}

pub fn configure() -> Config {
    let matches = clap_app!(Stat_Keys =>
            (version: "1.0")
            (author: "NAME <email>")
            (about: "decript.")

            (@arg RUN_MODE: +required +takes_value "either `client`, `server` or `single`")
            (@arg IP: -i --ip +takes_value "weefwfe")
            (@arg PORT: -p --port +takes_value "weefwfe")
            (@arg LOAD_PATH: -l --load_path +takes_value "weefwfe")
        ).get_matches();


    let run_mode = match matches.value_of("RUN_MODE").unwrap() {
        "client" => RunMode::ClientPlayer,
        "server" => RunMode::Server,
        "single" => RunMode::SinglePlayer,
        _ => panic!("NEED TO USE A VALID RUNMODE! SEE --help"),
    };

    Config{
        run_mode : run_mode,
        maybe_state : match matches.value_of("load_path") {
            Some(s) => Some(GameState::load_from(s).unwrap()),
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
