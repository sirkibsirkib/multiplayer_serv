use super::game_state;
use super::game_state::{Entity,Point};
use super::super::identity::{EntityID,LocationID};
use super::server_game_state::{LocationLoader,START_LOCATION};

use std::time::Duration;
use std::sync::{Arc,Mutex};
use std::time;
use std::collections::HashMap;

use super::super::network::messaging::{MsgToClientSet,MsgFromClient,MsgToClient,MsgToServer,Diff};
use super::super::network::{ProtectedQueue,UserBase};
use super::ClientID;
use std::thread;
use super::SaverLoader;

#[derive(Serialize,Deserialize,Debug)]
struct ServerData {
    next_eid : EntityID,
    cid_to_controlling : HashMap<ClientID, (EntityID,LocationID)>,
}

impl ServerData {
    fn use_next_eid(&mut self) -> EntityID {
        self.next_eid += 1;
        self.next_eid - 1
    }
}

pub fn game_loop(serv_in : Arc<ProtectedQueue<MsgFromClient>>,
                 serv_out : Arc<ProtectedQueue<MsgToClientSet>>,
                 userbase : Arc<Mutex<UserBase>>,
                 sl : SaverLoader,
             ) {
    println!("Server game loop");

    let mut server_data : ServerData = match sl.load_me("./server_data.lel") {
        Ok(x) => {
            println!("Successfully loaded server_data");
            x
        },
        Err(_) => {
            println!("Failed to load server_data. Made fresh");
            ServerData {
                next_eid : 0,
                cid_to_controlling : HashMap::new(),
            }
        }
    };

    // let mut global_state : GameState = GameState::new();

    //comment
    let time_between_updates = time::Duration::from_millis(1000/game_state::UPDATES_PER_SEC);

    // println!("TIME BETWEEN SYNCHFLOODS INFLATED FOR TESTING OK?", );
    let time_between_syncfloods = time::Duration::from_millis(3000);

    let mut location_loader = LocationLoader::new(Duration::new(10,0), sl.clone());

    let mut last_syncflood_at = time::Instant::now();

    // global_state.add_entity(8437, Entity::new(Point{x:0.9, y:0.9}));
    loop {
        let update_start = time::Instant::now();
        if last_syncflood_at.elapsed() > time_between_syncfloods {
            last_syncflood_at = update_start;
            synchflood(&serv_out, &location_loader);

            println!("SAVING FOR TESTING PURPOSES");
            let u : &UserBase = &userbase.lock().unwrap();
            sl.save_me(u, "user_base.lel").expect("couldn't save user base!");
            sl.save_me(&server_data, "./server_data.lel").expect("Couldn't save server_data");
            location_loader.save_all_locations();
            location_loader.unload_overdue_backgrounds();
            location_loader.print_status();
        }

        update_step(&serv_in, &serv_out, &mut location_loader, &userbase, &mut server_data);

        let since_update = update_start.elapsed();
        if since_update < time_between_updates {
            thread::sleep(time_between_updates - since_update);
        }
    }
}

fn synchflood(serv_out : &Arc<ProtectedQueue<MsgToClientSet>>, location_loader : &LocationLoader,) {
    // println!("SYNCHFLOOD!");
    //TODO dont send everything to everyone. instead figure out what each client can see
    // for (eid, e) in global_state.entity_iterator() {
    //     serv_out.lock_push_notify(
    //         MsgToClientSet::All(
    //             MsgToClient::EntityMoveTo(*eid, *e.p())
    //         )
    //     );
    // }
}

fn update_step(serv_in : &Arc<ProtectedQueue<MsgFromClient>>,
               serv_out : &Arc<ProtectedQueue<MsgToClientSet>>,
               location_loader : &mut LocationLoader,
               user_base : &Arc<Mutex<UserBase>>,
               server_data : &mut ServerData,
           ) {
    //comment
    let mut outgoing_updates : Vec<MsgToClientSet> = vec![];

    //fetch all requests and act appropriately
    if let Some(drained) = serv_in.impatient_drain() {
        for d in drained {
            match d.msg {
                MsgToServer::ControlMoveTo(lid,eid,pt) => {
                    if Some(&(eid,lid)) == server_data.cid_to_controlling.get(&d.cid) {
                        println!("You DO have permission to ctrl move that!");
                        location_loader.apply_diff_to(lid, Diff::MoveEntityTo(eid,pt),false);
                        outgoing_updates.push(
                            MsgToClientSet::All (
                                MsgToClient::GiveEntityData(eid,lid,pt),
                            )
                        );
                    } else {
                        println!("You don't have permission to ctrl move that!");
                    }
                }
                MsgToServer::RequestLocationData(lid) => {
                    location_loader.client_subscribe(d.cid, lid);
                    if let Some(&(eid,old_lid)) = server_data.cid_to_controlling.get(&d.cid) {
                        if lid != old_lid {
                            location_loader.client_unsubscribe(d.cid, old_lid);
                        }
                    }
                    let loc_prim = *location_loader.get_location_primitive(lid);
                    for (eid, pt) in location_loader.entity_iterator(lid) {
                        println!(">> informing client{:?} of eid {:?} {:?}", &d.cid, eid, pt);
                        outgoing_updates.push(
                            MsgToClientSet::Only(
                                MsgToClient::GiveLocationPrimitive(lid, loc_prim),
                                d.cid,
                            )
                        );
                        outgoing_updates.push(
                            MsgToClientSet::Only(
                                MsgToClient::GiveEntityData(*eid,lid,*pt),
                                d.cid,
                            )
                        );
                    }
                }
                MsgToServer::RequestControlling => {
                    if server_data.cid_to_controlling.get(&d.cid) == None {
                        println!("cid_to_controlling");
                        let mut locked_ub = user_base.lock().unwrap();
                        if ! locked_ub.client_is_setup(d.cid) {
                            println!("CLIENT {:?} having first-time setup", d.cid);
                            let player_eid = server_data.use_next_eid();
                            locked_ub.set_client_setup_true(d.cid);
                            server_data.cid_to_controlling.insert(d.cid, (player_eid,START_LOCATION));
                            location_loader.apply_diff_to(
                                START_LOCATION,
                                Diff::PlaceInside(player_eid,[10,10]),
                                true,
                            )
                        }
                    }
                    if let Some(&(eid,lid)) = server_data.cid_to_controlling.get(&d.cid) {
                        outgoing_updates.push(
                            MsgToClientSet::Only(
                                MsgToClient::GiveControlling(eid,lid),
                                d.cid,
                            )
                        );
                    } else {
                        panic!("WTFFFF");
                    }

                },
                x => {
                    println!("SERVER CAN'T HANDLE {:?}", &x);
                    unimplemented!();
                },
            }
        }
    }

    //TODO game tick here

    // push all resultant game updates to clients
    serv_out.lock_pushall_notify(outgoing_updates.drain(..));
}

fn inner_unwrap<T : Copy>(o : Option<&T>) -> Option<T> {
    if let Some(x) = o {
        Some(*x)
    } else {
        None
    }
}
