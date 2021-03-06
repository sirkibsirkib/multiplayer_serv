mod server_game_state;
mod server_resources;
mod subscription_manager;

use self::subscription_manager::SubscriptionManager;
use super::game_state;
use ::points::*;
use super::entities::{EntityData};
use super::super::identity::{EntityID,LocationID};
use self::server_game_state::{LocationLoader,START_LOCATION_LID};
use self::server_resources::ServerResources;
use rand::{Isaac64Rng,SeedableRng};

use std::sync::{Arc,Mutex};
use std::time;
use std::collections::HashMap;

use utils::traits::*;
use super::objects::{ObjectData};
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
impl KnowsSavePrefix for ServerData {
    fn get_save_prefix() -> String {
        "server_data".to_owned()
    }
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
    let mut subscription_manager = SubscriptionManager::new();
    let mut server_data : ServerData = match sl.load_without_key() {
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
    let mut sr = ServerResources::new(sl.clone(), Isaac64Rng::from_seed(&[3]));

    sr.define_object(0, ObjectData::new(0, 1.0));

    let time_between_updates = time::Duration::from_millis(1000/game_state::UPDATES_PER_SEC);

    // println!("TIME BETWEEN SYNCHFLOODS INFLATED FOR TESTING OK?", );
    let time_between_syncfloods = time::Duration::from_millis(3000);
    // let mut location_loader = LocationLoader::new(Duration::new(10,0), sl.clone());
    let mut last_syncflood_at = time::Instant::now();
    loop {
        let update_start = time::Instant::now();
        if last_syncflood_at.elapsed() > time_between_syncfloods {
            last_syncflood_at = update_start;
            synchflood(&serv_out, &mut sr);

            println!("SAVING FOR TESTING PURPOSES");
            let u : &UserBase = &userbase.lock().unwrap();
            sl.save_without_key(&server_data).expect("couldn't save server data!");
            sl.save_without_key(u).expect("couldn't save user base!");
            sr.save_all();
        }

        update_step(
            &serv_in,
            &serv_out,
            // &mut location_loader,
            &userbase,
            &mut server_data,
            &mut sr,
            &mut subscription_manager,
        );

        let since_update = update_start.elapsed();
        if since_update < time_between_updates {
            thread::sleep(time_between_updates - since_update);
        }
    }
}

fn synchflood(serv_out : &Arc<ProtectedQueue<MsgToClientSet>>, sr: &mut ServerResources) {
    //TODO send entity updates to all
}

fn update_step(serv_in : &Arc<ProtectedQueue<MsgFromClient>>,
               serv_out : &Arc<ProtectedQueue<MsgToClientSet>>,
               user_base : &Arc<Mutex<UserBase>>,
               server_data : &mut ServerData,
               sr : &mut ServerResources,
               subscription_manager: &mut SubscriptionManager,
           ) {
    //comment
    let mut outgoing_updates : Vec<MsgToClientSet> = vec![];

    //fetch all requests and act appropriately
    if let Some(drained) = serv_in.impatient_drain() {
        for d in drained {
            match d.msg {
                MsgToServer::ControlMoveTo(lid,eid,pt) => {
                    if Some(&(eid,lid)) == server_data.cid_to_controlling.get(&d.cid) {
                        println!("Ok you may move that!");
                        let diff = Diff::MoveEntityTo(eid,pt);
                        if sr.get_mut_location(lid).apply_diff(diff).is_ok() {
                            outgoing_updates.push(
                                MsgToClientSet::Subset (
                                    MsgToClient::ApplyLocationDiff(lid,diff),
                                    subscription_manager.get_subs_for(lid),
                                )
                            );
                        } else {
                            println!("CLIENT MOVE INHIBITED");
                        }
                    } else {
                        println!("You don't have permission to ctrl move that!");
                    }
                },
                MsgToServer::RequestObjectData(oid) => {
                    outgoing_updates.push(
                        MsgToClientSet::Only(
                            MsgToClient::GiveObjectData(oid, *sr.get_object(oid)),
                            d.cid,
                        )
                    );
                },
                MsgToServer::RequestEntityData(eid) => {
                    outgoing_updates.push(
                        MsgToClientSet::Only(
                            MsgToClient::GiveEntityData(eid, *sr.get_entity(eid)),
                            d.cid,
                        )
                    );
                },
                MsgToServer::ClientHasDisconnected => {
                    println!("Client {:?} has disconnected!", &d.cid);
                    user_base.lock().unwrap().logout(d.cid);
                    if let Some(&(_,old_lid)) = server_data.cid_to_controlling.get(&d.cid) {
                        subscription_manager.unsubscribe(old_lid, d.cid);
                    }
                },
                MsgToServer::RequestWorldData(wid) => {
                    outgoing_updates.push(
                        MsgToClientSet::Only(
                            MsgToClient::GiveWorldPrimitive(
                                wid,
                                sr.get_world_primitive(wid).clone(),
                            ),
                            d.cid,
                        )
                    );

                },
                MsgToServer::RequestLocationData(lid) => {
                    subscription_manager.subscribe(lid, d.cid);
                    // sr.borrow_mut_location_loader().client_subscribe(d.cid, lid);
                    if let Some(&(_,old_lid)) = server_data.cid_to_controlling.get(&d.cid) {
                        if lid != old_lid {
                            subscription_manager.unsubscribe(lid, d.cid);
                            // sr.borrow_mut_location_loader().client_unsubscribe(d.cid, old_lid);
                        }
                    }
                    outgoing_updates.push(
                        MsgToClientSet::Only(
                            MsgToClient::GiveLocationPrimitive(
                                lid,
                                sr.get_location_primitive(lid).clone()
                            ),
                            d.cid,
                        )
                    );
                    for (eid, pt) in sr.get_location(lid).entity_iterator() {
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
                            sr.define_entity(player_eid, EntityData::new(1, 0.7));
                            locked_ub.set_client_setup_true(d.cid);
                            server_data.cid_to_controlling.insert(d.cid, (player_eid,START_LOCATION_LID));
                            let free_pt : DPoint2 =
                                sr.get_location(START_LOCATION_LID)
                                .free_point()
                                .expect("Oh no! start loc is full. cant spawn");
                            let mk_diff = Diff::PlaceInside(player_eid,free_pt);
                            sr.get_mut_location(START_LOCATION_LID).apply_diff(mk_diff)
                            .expect("YOU SAID LOCATION WAS FREE");
                            outgoing_updates.push(
                                MsgToClientSet::Subset (
                                    MsgToClient::ApplyLocationDiff(START_LOCATION_LID,mk_diff),
                                    subscription_manager.get_subs_for(START_LOCATION_LID),
                                )
                            );
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
