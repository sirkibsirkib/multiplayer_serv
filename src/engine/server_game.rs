use super::game_state;
use super::game_state::GameState;
use std::sync::Arc;
use std::time;
use super::super::network::{ProtectedQueue,MsgFromClient,MsgToClientSet};
use std::thread;

pub fn game_loop(initial_state : Option<GameState>,
                 serv_in : Arc<ProtectedQueue<MsgFromClient>>,
                 serv_out : Arc<ProtectedQueue<MsgToClientSet>>) {
    //comment


    let time_between_updates = time::Duration::from_millis(1000/game_state::UPDATES_PER_SEC);
    let mut global_state : GameState = if let Some(s) = initial_state {
        s
    } else {
        GameState::new()
    };
    loop {
        let now = time::Instant::now();

        update_step(&serv_in, &serv_out, &mut global_state);

        let elapsed = now.elapsed();
        if elapsed < time_between_updates {
            thread::sleep(time_between_updates - elapsed);
        }
    }
}

fn update_step(serv_in : &Arc<ProtectedQueue<MsgFromClient>>,
               serv_out : &Arc<ProtectedQueue<MsgToClientSet>>,
               global_state : &mut GameState) {
    //comment

}
