

extern crate find_folder;
extern crate rand;
extern crate piston_window;
extern crate image;

mod view;
mod asset_manager;
mod cache_manager;
mod client_resources;

use self::client_resources::ClientResources;

use self::cache_manager::CacheManager;
// use std::collections::HashMap;
use self::asset_manager::{AssetManager,HardcodedAssets};
use self::view::{View,ViewPerspective};
use std::sync::{Arc};
use super::super::network::{ProtectedQueue};
use super::ClientID;
use super::super::network::messaging::{MsgToClient,MsgToServer};
use super::super::identity::*;
use std::time::{Instant,Duration};
use super::game_state::locations::{Location};
use super::game_state::worlds::{WorldPrimitive,World};
use super::entities::{EntityDataSet};
use super::objects::{ObjectDataSet};
use ::utils::traits::*;
use ::points::*;
use std::path::Path;
use ::saving::SaverLoader;

const WIDTH : f64 = 600.0;
const HEIGHT : f64 = 400.0;

use self::piston_window::*;
use super::game_state;
use ::points::*;


struct Timer {
    ins : Instant,
    setdur : Duration,
}

struct MyData {
    view : Option<View>,
    controlling : Option<(EntityID,LocationID)>,
    cid : ClientID,
    wid: WorldID,
    longitude: f64,
    viewing_map: bool,
}

pub struct Dataset {
    asset_manager : AssetManager,
    entity_dataset : EntityDataSet,
    object_dataset : ObjectDataSet,
    data_requests_supressed_until : Timer,
    outgoing_request_cache : Vec<MsgToServer>,
}

pub fn game_loop(client_in : Arc<ProtectedQueue<MsgToClient>>,
                 client_out : Arc<ProtectedQueue<MsgToServer>>,
                 cid : ClientID,
                 sl: SaverLoader,
             ) {

    let client_resources = ClientResources::new(sl.clone, client_out.clone(), Duration::from_millis(400));
    let mut window = init_window();
    let mut my_data = MyData {
        view: None,
        controlling: None,
        cid: cid,
        wid: 0,
        longitude: 0.0,
        viewing_map: false,
    };
    // let mut remote_info = RemoteInfo::new();

    // let mut cache_manager = CacheManager::new(sl.clone());
    let hardcoded_assets = HardcodedAssets::new(&mut window.factory);


    let mut dataset = Dataset {
        asset_manager : AssetManager::new(&window.factory, sl),
        entity_dataset : EntityDataSet::new(),
        object_dataset : ObjectDataSet::new(),
        data_requests_supressed_until : Timer {ins : Instant::now(),setdur : Duration::from_millis(500)},
        outgoing_request_cache : vec![],
    };

    dataset.outgoing_request_cache.push(
        MsgToServer::RequestControlling
    );
    println!("Client game loop");
    let mut holding : Option<Button> = None;

    let mut mouse_at : Option<[f64 ; 2]> = None;
    while let Some(e) = window.next() {
        if let Some(_) = e.render_args() {
            window.draw_2d(&e, | _ , graphics| clear([0.0; 4], graphics));
            if let Some(ref v) = my_data.view {
                View::clear_window(&e, &mut window);
                if my_data.viewing_map {
                    v.render_world(&e, &mut window, &mut dataset, my_data.wid, &hardcoded_assets, my_data.longitude);
                } else {
                    v.render_location(&e, &mut window, &mut dataset);
                }
            }
        }
        if let Some(q) = e.mouse_scroll_args() {
            if let Some(ref mut v) = my_data.view {
                if q[1] < 0.0 {
                    v.zoom_out();
                } else if q[1] > 0.0 {
                    v.zoom_in();
                }
            }
        }

        if let Some(z) = e.mouse_cursor_args() {
            mouse_at = Some(z);
        }
        if let Some(button) = e.press_args() {
            holding = Some(button);
        }


        if let Some(button) = e.release_args() {
            holding = None;
            if button == Button::Mouse(MouseButton::Left) {
                if_chain! {
                    if let Some(ref mut v) = my_data.view;
                    if let Some(m) = mouse_at;
                    if let Some((eid, _)) = my_data.controlling;
                    if let Some(pt) = v.translate_screenpt(CPoint2::new(m[0] as f32, m[1] as f32));
                    then {
                        dataset.outgoing_request_cache.push(
                            MsgToServer::ControlMoveTo(my_data.controlling.unwrap().1, eid, pt)
                        );
                    }
                }
            } else if button == Button::Keyboard(Key::M) {
                my_data.viewing_map = !my_data.viewing_map;
            }
        }

        if let Some(_) = e.update_args() {
            //Holding key
            if let Some(holding_thing) = holding {
                if holding_thing == Button::Keyboard(Key::D) {
                    my_data.longitude += 0.004;
                    if my_data.longitude >= 1.0 {my_data.longitude -= 1.0}
                } else if holding_thing == Button::Keyboard(Key::A) {
                    my_data.longitude -= 0.004;
                    if my_data.longitude < 0.0 {my_data.longitude += 1.0}
                }
            }
            //SYNCHRONIZE!
            synchronize(
                &client_in,
                &client_out,
                &mut my_data,
                &mut dataset,
                &mut cache_manager,
            );
        }
    }
}

fn synchronize(client_in : &Arc<ProtectedQueue<MsgToClient>>,
               client_out : &Arc<ProtectedQueue<MsgToServer>>,
               // outgoing_update_requests : &mut Vec<MsgToServer>,
               my_data : &mut MyData,
               dataset : &mut Dataset,
               cache_manager : &mut CacheManager,
               // entity_data : &mut EntityDataSet,
              ) {
    //comment
    if let Some(drained) = client_in.impatient_drain() {
        //these are all updates from the server
        for d in drained {
            use MsgToClient::*;
            match d {
                GiveWorldPrimitive(wid, world_prim) => {
                    cache_manager.cache_world_primitive(wid, world_prim);
                    cache_manager.ensure_map_file_exists_for(wid, &dataset.asset_manager);
                },
                GiveObjectData(oid,data) => {
                    dataset.object_dataset.insert(oid,data);
                },
                GiveEntityData(eid,data) => {
                    dataset.entity_dataset.insert(eid,data);
                },
                ApplyLocationDiff(lid,diff) => {
                    if let Some(ref mut view) = my_data.view {
                        if let Some((c_eid, c_lid)) = my_data.controlling {
                            if c_lid == lid {
                                view.get_location_mut().apply_diff(diff);
                            }
                        }
                    }
                },
                GiveLocationPrimitive(lid, loc_prim) => {
                    let wid = loc_prim.wid;
                    println!("OK got loc prim from serv");
                    if cache_manager.world_is_cached(wid) {
                        let w = cache_manager.get_world(wid).expect("wtf cachebro");
                        if let Some((c_eid, c_lid)) = my_data.controlling {
                            let zone = w.get_zone(loc_prim.zone_id).clone();
                            if c_lid == lid {
                                println!("... and I am expecting it");
                                my_data.view = Some(View::new(
                                    my_data.controlling.unwrap().0,
                                    Location::generate_new(loc_prim, zone),
                                    // Location::new(loc_prim),
                                    ViewPerspective::DEFAULT_SURFACE,
                                ));
                            }
                        }
                    } else {
                        dataset.outgoing_request_cache.push(
                            MsgToServer::RequestWorldData(wid)
                        );
                        //resend this plz
                        dataset.outgoing_request_cache.push(
                            MsgToServer::RequestLocationData(lid)
                        );
                    }
                },
                GiveControlling(eid, lid) => {
                    let mut going_to_new_loc = false;
                    if let Some((_, my_lid)) = my_data.controlling {
                        // I am already controlling something!
                        if my_lid != lid {
                            // new location!
                            going_to_new_loc = true;
                        }
                    } else {
                        // I am controlling nothing!
                        going_to_new_loc = true;
                    }
                    my_data.controlling = Some((eid,lid));
                    if going_to_new_loc {
                        my_data.view = None; //subsequent message will populate this
                        dataset.outgoing_request_cache.push(
                            MsgToServer::RequestLocationData(lid)
                        ); // request data to populate `my_data.viewing`
                    }
                },
                _ => {
                    println!("Client engine got msg {:?} and didn't know how to deal", d);
                    unimplemented!();
                },
            }
        }
    }
    if ! dataset.outgoing_request_cache.is_empty() {
        client_out.lock_pushall_notify(dataset.outgoing_request_cache.drain(..));
    }
}

fn am_controlling(eid : EntityID, my_data : &MyData) -> bool {
    if let Some((cntl_eid, _)) = my_data.controlling {
        cntl_eid == eid
    } else {
        false
    }
}



fn init_window() -> PistonWindow {
    let mut window: PistonWindow = WindowSettings::new("Multiplayer", ((WIDTH) as u32, (HEIGHT) as u32))
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| { panic!("Failed to build PistonWindow: {}", e) });

    let event_settings = EventSettings {
        max_fps: 32,
        ups: game_state::UPDATES_PER_SEC,
        ups_reset: 2,
        swap_buffers: true,
        bench_mode: false,
        lazy: false,
    };
    window.set_event_settings(event_settings);
    window
}
