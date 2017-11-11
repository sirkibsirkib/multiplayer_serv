mod view;
mod asset_manager;

use self::asset_manager::AssetManager;
use self::view::{View,ViewPerspective};

use std::sync::Arc;
use super::super::network::{ProtectedQueue};
use super::ClientID;
use super::super::network::messaging::{MsgToClient,MsgToServer};
use super::super::identity::{LocationID,EntityID,AssetID};
use std::collections::HashMap;
use std::time::{Instant,Duration};

extern crate piston_window;
use self::piston_window::*;
extern crate rand;
// use self::rand::{SeedableRng, Rng, Isaac64Rng};
// use self::rand::{SeedableRng, Rng, Isaac64Rng};
use super::game_state::{Location};
use super::entities::{EntityData,EntityDataSet};

extern crate find_folder;

const WIDTH : f64 = 500.0;
const HEIGHT : f64 = 400.0;



const E_DATA_SUPPRESS_DUR : Duration = Duration::from_secs(3); //TODO

use super::game_state;
use super::game_state::{Point};
use super::procedural::NoiseMaster;

struct InstantHolder {
    ins : Instant,
}

struct MyData {
    view : Option<View>,
    controlling : Option<(EntityID,LocationID)>,
    cid : ClientID,
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

    let nm = NoiseMaster::new();
    // let nf0 = nm.generate_noise_field([1,2,3,4,4], [1.0,0.3,0.2,0.2,0.1], 1.0);
    // for i in 0..20 {
    //     println!("''{:?}", nf0.sample([0,i]));
    // }

    let mut asset_manager = AssetManager::new(&window.factory);
    let mut entity_data = EntityDataSet::new();

    let mut entity_data_suppressed_until : InstantHolder = InstantHolder{ins : Instant::now()};


    // let assets = find_folder::Search::ParentsThenKids(3, 3)
    //     .for_folder("assets").unwrap();
    // let test_path = assets.join("test.png");
    // let rust_logo: G2dTexture = Texture::from_path(
    //         &mut window.factory,
    //         &test_path,
    //         Flip::None,
    //         &TextureSettings::new()
    //     ).unwrap();
    // this is just a local vector to batch requests. populating this essentially populates client_out
    let mut outgoing_update_requests : Vec<MsgToServer> = vec![];

    outgoing_update_requests.push(
        MsgToServer::RequestControlling
    );
    println!("Client game loop");
    let mut mouse_at : Option<[f64 ; 2]> = None;
    while let Some(e) = window.next() {

        if let Some(_) = e.render_args() {
            window.draw_2d(&e, | _ , graphics| clear([0.0; 4], graphics));
            render_location(
                &e,
                &mut window,
                &mut my_data,
                &mut outgoing_update_requests,
                &mut asset_manager,
                &entity_data,
                &mut entity_data_suppressed_until,
            );
            // window.draw_2d(&e, |c, g| {
            //     image(&rust_logo, c.transform, g);
            // });
        }
        if let Some(z) = e.mouse_cursor_args() {
            mouse_at = Some(z);
        }
        if let Some(button) = e.release_args() {
            if button == Button::Mouse(MouseButton::Left) {
                if let Some(ref mut v) = my_data.view {
                    if let Some(m) = mouse_at {
                        if let Some((eid, _)) = my_data.controlling {
                            outgoing_update_requests.push(
                                MsgToServer::ControlMoveTo(
                                    my_data.controlling.unwrap().1,
                                    eid,
                                    v.translate_screenpt(m),
                                )
                            );
                        }
                    }
                }
            }
        }

        if let Some(_) = e.update_args() {
            //SYNCHRONIZE!
            synchronize(
                &client_in,
                &client_out,
                &mut outgoing_update_requests,
                &mut my_data,
                &nm,
                &mut entity_data,
            );
        }
    }
}

fn synchronize(client_in : &Arc<ProtectedQueue<MsgToClient>>,
               client_out : &Arc<ProtectedQueue<MsgToServer>>,
               outgoing_update_requests : &mut Vec<MsgToServer>,
               my_data : &mut MyData,
               nm : &NoiseMaster,
               entity_data : &mut EntityDataSet,
              ) {
    //comment
    if let Some(drained) = client_in.impatient_drain() {
        //these are all updates from the server
        for d in drained {
            use MsgToClient::*;
            match d {
                GiveEntityData(eid,data) => {
                    entity_data.insert(eid,data);
                },
                ApplyLocationDiff(lid,diff) => {
                    if let Some(ref mut view) = my_data.view {
                        if let Some((c_eid, c_lid)) = my_data.controlling {
                            if c_lid == lid {
                                view.get_location_mut().apply_diff(diff);
                            }
                        }
                    }
                }
                GiveLocationPrimitive(lid, loc_prim) => {
                    println!("OK got loc prim from serv");
                    if let Some((c_eid, c_lid)) = my_data.controlling {
                        if c_lid == lid {
                            println!("... and I am expecting it");
                            my_data.view = Some(View::new(
                                my_data.controlling.unwrap().0,
                                Location::new(loc_prim, nm),
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
                        outgoing_update_requests.push(
                            MsgToServer::RequestLocationData(lid)
                        ); // request data to populate `my_data.viewing`
                    }
                }
                _ => {
                    println!("Client engine got msg {:?} and didn't know how to deal", d);
                    unimplemented!();
                },
            }
        }
    }
    if ! outgoing_update_requests.is_empty() {
        client_out.lock_pushall_notify(outgoing_update_requests.drain(..));
    }
}

fn am_controlling(eid : EntityID, my_data : &MyData) -> bool {
    if let Some((cntl_eid, _)) = my_data.controlling {
        cntl_eid == eid
    } else {
        false
    }
}

fn render_location<E>(event : &E,
                   window : &mut PistonWindow,
                   my_data : &mut MyData,
                   outgoing_update_requests : &mut Vec<MsgToServer>,
                   asset_manager : &mut AssetManager,
                   entity_data : & EntityDataSet,
                   entity_data_suppressed_until : &mut InstantHolder,
               ) where E : GenericEvent {
    if let Some(ref v) = my_data.view {
        let mut missing_eid_assets = vec![];
        for (eid, pt) in v.get_location().entity_iterator() {
            if missing_eid_assets.contains(&eid) {
                //already did this dance. waiting for it to arrive
                continue;
            }
            if let Some(ent_data) = entity_data.get(*eid) {
                let tex = asset_manager.get_texture_for(ent_data.aid);
                // let aid = ;
                let col = if am_controlling(*eid, &my_data) {
                    [0.0, 1.0, 0.0, 1.0] //green
                } else {
                    [0.7, 0.7, 0.7, 1.0] //gray
                };
                let rad = 10.0;
                let screen_pt = v.translate_pt(*pt);
                window.draw_2d(event, |c, g| {
                    image(tex, c.transform, g);
                });
                let el = [
                    screen_pt[0] - rad,
                    screen_pt[1] - rad,
                    rad*2.0,
                    rad*2.0
                ];
                // println!("client sees eid {:?} ellipse {:?}", &eid, &el);
                window.draw_2d(event, |context, graphics| {
                            ellipse(
                                col,
                                el,
                                context.transform,
                                graphics
                          );
                      }
                );
            } else {
                missing_eid_assets.push(eid);
                continue;
            }
        }
        if ! missing_eid_assets.is_empty() {
            let now = Instant::now();
            if entity_data_suppressed_until.ins < now {
                for eid in missing_eid_assets {
                    outgoing_update_requests.push(
                        MsgToServer::RequestEntityData(*eid)
                    ); // request to populate asset manager
                }
                entity_data_suppressed_until.ins = now + E_DATA_SUPPRESS_DUR;
            }
        }
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
