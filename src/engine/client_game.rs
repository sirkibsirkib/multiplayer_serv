use std::sync::Arc;
use super::super::network::{ProtectedQueue,ClientID};
use super::super::network::messaging::{MsgToClient,MsgToServer};

extern crate piston_window;
use self::piston_window::*;
extern crate rand;
use self::rand::{SeedableRng, Rng, Isaac64Rng};
use super::game_state::{Location,LocationID};
use std::collections::HashMap;


extern crate find_folder;

const WIDTH : f64 = 500.0;
const HEIGHT : f64 = 400.0;

use super::game_state;
use super::game_state::{Point,EntityID,Entity};

struct MyData2 {
    current_lid : LocationID,
    subscriptions : HashMap<LocationID, Location>,
    
}

struct MyData {

    lid : Option<LocationID>,
    viewing : Option<Location>,
    controlling : Option<(EntityID,LocationID)>,
    cid : ClientID,
}


pub fn game_loop(client_in : Arc<ProtectedQueue<MsgToClient>>,
                 client_out : Arc<ProtectedQueue<MsgToServer>>,
                 cid : ClientID) {
    let mut window = init_window();
    let mut my_data = MyData{
        lid : None,
        viewing : None,
        controlling : None,
        cid : cid,
    };


    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    let test_path = assets.join("test.png");
    let rust_logo: G2dTexture = Texture::from_path(
            &mut window.factory,
            &test_path,
            Flip::None,
            &TextureSettings::new()
        ).unwrap();

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
            render_location(&e, &mut window, &mut my_data, );
            window.draw_2d(&e, |c, g| {
                image(&rust_logo, c.transform, g);
            });
        }
        if let Some(z) = e.mouse_cursor_args() {
            mouse_at = Some(z);
        }
        if let Some(button) = e.release_args() {
            if button == Button::Mouse(MouseButton::Left) {
                if let Some(m) = mouse_at {
                    if let Some((eid, _)) = my_data.controlling {
                        let p = Point {x:m[0]/WIDTH, y:m[1]/HEIGHT};
                        outgoing_update_requests.push(
                            MsgToServer::ControlMoveTo(my_data.controlling.unwrap().1, eid, p)
                        );
                    }
                }
            }
        }

        if let Some(_) = e.update_args() {
            //SYNCHRONIZE!
            synchronize(&client_in, &client_out, &mut outgoing_update_requests, &mut my_data);
        }
    }
}

fn synchronize(client_in : &Arc<ProtectedQueue<MsgToClient>>,
               client_out : &Arc<ProtectedQueue<MsgToServer>>,
               outgoing_update_requests : &mut Vec<MsgToServer>,
               my_data : &mut MyData,
              ) {
    //comment
    if let Some(drained) = client_in.impatient_drain() {
        //these are all updates from the server
        for d in drained {
            use MsgToClient::*;
            match d {
                GiveEntityData(eid, lid, pt) => {
                    if let Some(ref mut loc) = my_data.viewing {
                        loc.place_inside(eid, Entity::new(pt));
                    }
                },
                EntityMoveTo(eid,pt) => {
                    if let Some(ref mut loc) = my_data.viewing {
                        loc.entity_move_to(eid,pt);
                    }
                },
                GiveControlling(maybe_eid_and_lid) => {
                    if my_data.controlling != maybe_eid_and_lid {
                        if let Some(x) = maybe_eid_and_lid {
                            let mut need_to_load = false;
                            if let Some((_, b)) = my_data.controlling{
                                if x.1 != b {
                                    need_to_load = true;
                                }
                            } else {
                                need_to_load = true;
                            }
                            if need_to_load {
                                //change location
                                my_data.viewing = Some(Location::new());
                                outgoing_update_requests.push(
                                    MsgToServer::RequestLocationData(x.1)
                                );
                            }
                            my_data.controlling = Some(x);
                        } else {
                            my_data.controlling = None;
                            my_data.viewing = None;
                        }
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

fn render_location(event : &Event, window : &mut PistonWindow, my_data : &mut MyData) {
    if let Some(ref loc) = my_data.viewing {
        for (eid, e) in loc.entity_iterator() {
            let col = if am_controlling(*eid, &my_data) {
                [0.0, 1.0, 0.0, 1.0] //green
            } else {
                [0.7, 0.7, 0.7, 1.0] //gray
            };
            let rad = 10.0;
            window.draw_2d(event, |context, graphics| {
                        ellipse(
                            col,
                            [
                                (e.p().x as f64)*WIDTH - rad,
                                (e.p().y as f64)*HEIGHT - rad,
                                rad*2.0,
                                rad*2.0
                            ],
                            context.transform,
                            graphics
                      );
                  }
            );
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
