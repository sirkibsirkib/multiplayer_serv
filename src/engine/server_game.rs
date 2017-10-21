use super::game_state;
use super::game_state::{GameState,EntityID,Entity,Point};
use std::sync::Arc;
use std::time;
use std::collections::HashMap;
use super::super::network::{ProtectedQueue,MsgFromClient,MsgToClientSet,ClientID,MsgToClient,MsgToServer};
use std::thread;

pub fn game_loop(mut global_state : GameState,
                 serv_in : Arc<ProtectedQueue<MsgFromClient>>,
                 serv_out : Arc<ProtectedQueue<MsgToClientSet>>) {
    println!("Server game loop");
    //comment
    let time_between_updates = time::Duration::from_millis(1000/game_state::UPDATES_PER_SEC);
    let mut player_controlling : HashMap<ClientID,Vec<EntityID>> = HashMap::new();

    global_state.add_entity(8437, Entity::new(Point{x:0.9, y:0.9}));

    loop {
        let now = time::Instant::now();

        update_step(&serv_in, &serv_out, &mut global_state, &mut player_controlling);

        let elapsed = now.elapsed();
        if elapsed < time_between_updates {
            thread::sleep(time_between_updates - elapsed);
        }
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
                MsgToServer::StartHandshake(_)=> {
                    //ignore
                }
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
