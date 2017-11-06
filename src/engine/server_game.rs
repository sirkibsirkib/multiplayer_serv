use super::game_state;
use super::game_state::{GameState,EntityID,Entity,Point};
use super::locations::{Location,LocationLoader};

use std::time::Duration;
use std::sync::{Arc,Mutex};
use std::time;
use std::collections::HashMap;
use super::super::network::{ProtectedQueue,MsgFromClient,MsgToClientSet,ClientID,MsgToClient,MsgToServer,UserBase};
use std::thread;
use super::SaverLoader;


pub fn game_loop(serv_in : Arc<ProtectedQueue<MsgFromClient>>,
                 serv_out : Arc<ProtectedQueue<MsgToClientSet>>,
                 userbase : Arc<Mutex<UserBase>>,
                 sl : SaverLoader,
             ) {
    println!("Server game loop");

    let mut global_state : GameState = GameState::new();

    //comment
    let time_between_updates = time::Duration::from_millis(1000/game_state::UPDATES_PER_SEC);

    // println!("TIME BETWEEN SYNCHFLOODS INFLATED FOR TESTING OK?", );
    let time_between_syncfloods = time::Duration::from_millis(3000);

    let mut player_controlling : HashMap<ClientID,Vec<EntityID>> = HashMap::new();

    let mut location_loader = LocationLoader::new(Duration::new(10,0), sl.clone());


    let mut last_syncflood_at = time::Instant::now();

    global_state.add_entity(8437, Entity::new(Point{x:0.9, y:0.9}));
    loop {
        let update_start = time::Instant::now();
        if last_syncflood_at.elapsed() > time_between_syncfloods {
            last_syncflood_at = update_start;
            synchflood(&serv_out, &global_state);

            println!("SAVING FOR TESTING PURPOSES");
            let u : &UserBase = &userbase.lock().unwrap();
            sl.save_me(u, "user_base.lel").expect("couldn't save user base!");
        }

        update_step(&serv_in, &serv_out, &mut global_state, &mut player_controlling);

        let since_update = update_start.elapsed();
        if since_update < time_between_updates {
            thread::sleep(time_between_updates - since_update);
        }
    }
}

fn synchflood(serv_out : &Arc<ProtectedQueue<MsgToClientSet>>, global_state : &GameState) {
    println!("SYNCHFLOOD!");
    //TODO dont send everything to everyone. instead figure out what each client can see
    for (eid, e) in global_state.entity_iterator() {
        serv_out.lock_push_notify(
            MsgToClientSet::All(
                MsgToClient::EntityMoveTo(*eid, *e.p())
            )
        );
    }
}

fn update_step(serv_in : &Arc<ProtectedQueue<MsgFromClient>>,
               serv_out : &Arc<ProtectedQueue<MsgToClientSet>>,
               global_state : &mut GameState,
               player_controlling : &mut HashMap<ClientID,Vec<EntityID>>) {
    //comment
    let mut outgoing_updates : Vec<MsgToClientSet> = vec![];

    //fetch all requests and act appropriately
    if let Some(drained) = serv_in.impatient_drain() {
        for d in drained {
            match d.msg {
                MsgToServer::LoadEntities => {
                    for e in global_state.entity_iterator() {
                        outgoing_updates.push(
                            MsgToClientSet::Only(MsgToClient::CreateEntity(*e.0,*e.1.p()), d.cid)
                        );
                    }
                }
                MsgToServer::RequestControlOf(eid) => {
                    try_add_control(d.cid, eid, global_state, player_controlling);
                    outgoing_updates.push(
                        MsgToClientSet::Only(MsgToClient::YouNowControl(eid), d.cid)
                    );
                }
                MsgToServer::CreateEntity(eid,pt) => {
                    global_state.add_entity(eid, Entity::new(pt));
                    outgoing_updates.push(
                        MsgToClientSet::All(MsgToClient::CreateEntity(eid,pt))
                    );

                }
                MsgToServer::ControlMoveTo(eid,pt) => {
                    if client_controls(d.cid, eid, player_controlling) {
                        global_state.entity_move_to(eid, pt);
                        outgoing_updates.push(
                            MsgToClientSet::All(MsgToClient::EntityMoveTo(eid, pt))
                        );
                    }
                }
                _ => {unimplemented!();}
            }
        }
    }

    //TODO game tick here

    // push all resultant game updates to clients
    serv_out.lock_pushall_notify(outgoing_updates.drain(..));
}

fn client_controls(cid : ClientID, eid : EntityID, player_controlling : &mut HashMap<ClientID,Vec<EntityID>>) -> bool {
    if let Some(controlling_list) = player_controlling.get(&cid) {
        controlling_list.contains(&eid)
    } else {
        false
    }
}

fn try_add_control(cid : ClientID,
                   eid : EntityID,
                   global_state : &mut GameState,
                   player_controlling : &mut HashMap<ClientID,Vec<EntityID>>) {
    if global_state.entity_exists(eid) {
        if let Some(controlling_list) = player_controlling.get_mut(&cid) {
            if ! controlling_list.contains(&eid) {
                controlling_list.push(eid);
            }
            return;
        }
        player_controlling.insert(cid, vec![eid]);
    }
}
