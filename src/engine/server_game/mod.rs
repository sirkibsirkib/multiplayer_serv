mod server_game_state;

use super::game_state;
use super::game_state::{Point};
use super::entities::{EntityData,EntityDataSet};
use super::super::identity::{EntityID,LocationID};
use self::server_game_state::{LocationLoader,START_LOCATION_LID};

use std::time::Duration;
use std::sync::{Arc,Mutex};
use std::time;
use std::collections::HashMap;

use ::network::messaging::{MsgToClientSet,MsgFromClient,MsgToClient,MsgToServer,Diff};
use ::network::{ProtectedQueue};
use ::network::userbase::{UserBase};
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
    let mut entity_data_set = match sl.load_me("./entity_data_set.lel") {
        Ok(x) => {
            println!("Successfully loaded entity data");
            x
        },
        Err(_) => {
            println!("Failed to load entity_data_set. Made fresh");
            EntityDataSet::new()
        }
    };

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
            sl.save_me(&entity_data_set, "./entity_data_set.lel").expect("couldn't save entity data!");
            sl.save_me(&server_data, "./server_data.lel").expect("Couldn't save server_data");
            location_loader.unload_overdue_backgrounds();
            location_loader.save_all_locations();
            location_loader.print_status();
        }

        update_step(&serv_in, &serv_out, &mut location_loader, &userbase, &mut server_data, &mut entity_data_set);

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
               entity_data_set : &mut EntityDataSet,
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
                        let diff = Diff::MoveEntityTo(eid,pt);
                        if location_loader.apply_diff_to(lid, diff,false).is_ok() {
                            outgoing_updates.push(
                                MsgToClientSet::Subset (
                                    MsgToClient::ApplyLocationDiff(lid,diff),
                                    location_loader.get_subscriptions_for(lid),
                                )
                            );
                        } else {
                            println!("CLIENT MOVE INHIBITED");
                        }
                    } else {
                        println!("You don't have permission to ctrl move that!");
                    }
                },
                MsgToServer::RequestEntityData(eid) => {
                    if let Some(data) = entity_data_set.get(eid) {
                        outgoing_updates.push(
                            MsgToClientSet::Only(
                                MsgToClient::GiveEntityData(eid, *data),
                                d.cid,
                            )
                        );
                    } else {
                        println!("Client asking for nonexistant entity data for eid {:?}", eid);
                        println!("entity data is actually {:?}", &entity_data_set);
                    }
                },
                MsgToServer::ClientHasDisconnected => {
                    println!("Client {:?} has disconnected!", &d.cid);
                    user_base.lock().unwrap().logout(d.cid);
                    if let Some(&(_,old_lid)) = server_data.cid_to_controlling.get(&d.cid) {
                        location_loader.client_unsubscribe(d.cid, old_lid);
                    }
                },
                MsgToServer::RequestLocationData(lid) => {
                    location_loader.client_subscribe(d.cid, lid);
                    if let Some(&(_,old_lid)) = server_data.cid_to_controlling.get(&d.cid) {
                        if lid != old_lid {
                            location_loader.client_unsubscribe(d.cid, old_lid);
                        }
                    }
                    let loc_prim = *location_loader.borrow_location(lid).get_location_primitive();
                    outgoing_updates.push(
                        MsgToClientSet::Only(
                            MsgToClient::GiveLocationPrimitive(lid, loc_prim),
                            d.cid,
                        )
                    );
                    for (eid, pt) in location_loader.borrow_location(lid).entity_iterator() {
                        println!(">> informing client{:?} of eid {:?} {:?}", &d.cid, eid, pt);
                        outgoing_updates.push(
                            MsgToClientSet::Only(
                                MsgToClient::ApplyLocationDiff(lid,Diff::PlaceInside(*eid,*pt)),
                                // MsgToClient::GiveEntityData(*eid,lid,*pt),
                                d.cid,
                            )
                        );
                    }
                },
                MsgToServer::RequestControlling => {
                    if server_data.cid_to_controlling.get(&d.cid) == None {
                        println!("cid_to_controlling");
                        let mut locked_ub = user_base.lock().unwrap();
                        if ! locked_ub.client_is_setup(d.cid) {
                            println!("CLIENT {:?} having first-time setup", d.cid);
                            let player_eid = server_data.use_next_eid();
                            entity_data_set.insert(player_eid, EntityData::new(0));
                            locked_ub.set_client_setup_true(d.cid);
                            server_data.cid_to_controlling.insert(d.cid, (player_eid,START_LOCATION_LID));
                            let free_pt : Point =
                                location_loader
                                .borrow_location(START_LOCATION_LID)
                                .free_point()
                                .expect("Oh no! start loc is full. cant spawn");
                            location_loader.apply_diff_to(
                                START_LOCATION_LID,
                                Diff::PlaceInside(player_eid,free_pt),
                                true,
                            ).expect("YOU SAID LOCATION WAS FREE");
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
