

extern crate find_folder;
extern crate rand;
extern crate piston_window;
extern crate image;

mod view;
mod asset_manager;

use self::asset_manager::AssetManager;
use self::view::{View,ViewPerspective};
use std::sync::{Arc};
use super::super::network::{ProtectedQueue};
use super::ClientID;
use super::super::network::messaging::{MsgToClient,MsgToServer};
use super::super::identity::{LocationID,EntityID,ObjectID};
use std::time::{Instant,Duration};
use super::game_state::locations::{Location};
use super::entities::{EntityDataSet};
use super::objects::{ObjectDataSet};
use super::primitives::*;
use ::points::*;

const WIDTH : f64 = 500.0;
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
}

struct Dataset {
    asset_manager : AssetManager,
    entity_dataset : EntityDataSet,
    object_dataset : ObjectDataSet,
    data_requests_supressed_until : Timer,
    outgoing_request_cache : Vec<MsgToServer>,
}

pub fn game_loop(client_in : Arc<ProtectedQueue<MsgToClient>>,
                 client_out : Arc<ProtectedQueue<MsgToServer>>,
                 cid : ClientID) {
    let mut window = init_window();
    let mut my_data = MyData {
        view : None,
        controlling : None,
        cid : cid,
    };
    // let mut remote_info = RemoteInfo::new();

    let mut dataset = Dataset {
        asset_manager : AssetManager::new(&window.factory),
        entity_dataset : EntityDataSet::new(),
        object_dataset : ObjectDataSet::new(),
        data_requests_supressed_until : Timer {ins : Instant::now(),setdur : Duration::from_millis(500)},
        outgoing_request_cache : vec![],
    };

    dataset.outgoing_request_cache.push(
        MsgToServer::RequestControlling
    );
    println!("Client game loop");

    let mut mouse_at : Option<[f64 ; 2]> = None;
    while let Some(e) = window.next() {
        if let Some(_) = e.render_args() {
            window.draw_2d(&e, | _ , graphics| clear([0.0; 4], graphics));
            if let Some(ref v) = my_data.view {
                v.render_location(&e, &mut window, &mut dataset);
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
        if let Some(button) = e.release_args() {
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
            }
        }

        if let Some(_) = e.update_args() {
            //SYNCHRONIZE!
            synchronize(
                &client_in,
                &client_out,
                &mut my_data,
                &mut dataset,
            );
        }
    }
}

fn synchronize(client_in : &Arc<ProtectedQueue<MsgToClient>>,
               client_out : &Arc<ProtectedQueue<MsgToServer>>,
               // outgoing_update_requests : &mut Vec<MsgToServer>,
               my_data : &mut MyData,
               dataset : &mut Dataset,
               // entity_data : &mut EntityDataSet,
              ) {
    //comment
    if let Some(drained) = client_in.impatient_drain() {
        //these are all updates from the server
        for d in drained {
            use MsgToClient::*;
            match d {
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
                    println!("OK got loc prim from serv");
                    if let Some((c_eid, c_lid)) = my_data.controlling {
                        if c_lid == lid {
                            println!("... and I am expecting it");
                            my_data.view = Some(View::new(
                                my_data.controlling.unwrap().0,
                                loc_prim.generate_new(),
                                // Location::new(loc_prim),
                                ViewPerspective::DEFAULT_SURFACE,
                            ));
                        }
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
