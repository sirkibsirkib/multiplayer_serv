use std::sync::Arc;
use super::super::network::{ProtectedQueue,MsgToClient,MsgToServer,ClientID};

extern crate piston_window;
use self::piston_window::*;
extern crate rand;
use self::rand::{SeedableRng, Rng, Isaac64Rng};


extern crate find_folder;

const WIDTH : f64 = 500.0;
const HEIGHT : f64 = 400.0;

use super::game_state;
use super::game_state::{GameState,Point,EntityID,Entity};


pub fn game_loop(client_in : Arc<ProtectedQueue<MsgToClient>>,
                 client_out : Arc<ProtectedQueue<MsgToServer>>,
                 c_id : ClientID) {
    let mut window = init_window();


    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    let test_path = assets.join("test.png");
    let rust_logo: G2dTexture = Texture::from_path(
            &mut window.factory,
            &test_path,
            Flip::None,
            &TextureSettings::new()
        ).unwrap();

    let mut next_eid = (c_id as u64 + 1) << 40; // this gives the server +40 bits of reserved eids

    let mut local_state = GameState::new();

    // this is just a local vector to batch requests. populating this essentially populates client_out
    let mut outgoing_update_requests : Vec<MsgToServer> = vec![];

    // to help it know which eid the user is attempting to manipulate with movement
    let mut controlling : Option<EntityID> = None;
    let mut r = Isaac64Rng::from_seed(&[c_id as u64]);

    outgoing_update_requests.push(
        MsgToServer::LoadEntities
    );
    println!("Client game loop");
    println!("my seed is {}", c_id);
    outgoing_update_requests.push(
        MsgToServer::CreateEntity(next_eid, Point{x:r.gen::<f64>(), y:r.gen::<f64>()})
    );
    outgoing_update_requests.push(
        MsgToServer::RequestControlOf(next_eid)
    );
    next_eid += 1;

    let mut mouse_at : Option<[f64 ; 2]> = None;
    while let Some(e) = window.next() {

        if let Some(_) = e.render_args() {
            window.draw_2d(&e, | _ , graphics| clear([0.0; 4], graphics));
            render_entities(&local_state, &e, &mut window, &controlling);
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
                    if let Some(eid) = controlling {
                        let p = Point {x:m[0]/WIDTH, y:m[1]/HEIGHT};
                        outgoing_update_requests.push(
                            MsgToServer::ControlMoveTo(eid, p)
                        );
                    }
                }
            }
        }

        if let Some(_) = e.update_args() {
            //SYNCHRONIZE!
            synchronize(&client_in, &client_out, &mut outgoing_update_requests, &mut controlling, &mut local_state);
        }
    }
}

fn synchronize(client_in : &Arc<ProtectedQueue<MsgToClient>>,
               client_out : &Arc<ProtectedQueue<MsgToServer>>,
               outgoing_update_requests : &mut Vec<MsgToServer>,
               controlling : &mut Option<EntityID>,
               local_state : &mut GameState) {
    //comment
    if ! outgoing_update_requests.is_empty() {
        client_out.lock_pushall_notify(outgoing_update_requests.drain(..));
    }
    if let Some(drained) = client_in.impatient_drain() {
        //these are all updates from the server
        for d in drained {
            use MsgToClient::*;
            match d {
                RefuseHandshake => {panic!()}
                CompleteHandshake(_) => {},
                CreateEntity(eid,pt) => {
                    local_state.add_entity(eid,Entity::new(pt))
                },
                YouNowControl(eid) => {*controlling = Some(eid)},
                YouNoLongerControl(eid) => {
                    if let &mut Some(existing_eid) = controlling {
                        if existing_eid == eid {
                            *controlling = None;
                        }
                    }
                },
                EntityMoveTo(eid,pt) => {
                    local_state.entity_move_to(eid,pt);
                }
            }
        }
    }
}

fn render_entities(game_state : &GameState, event : &Event, window : &mut PistonWindow, controlling : &Option<EntityID>) {
    for (eid, e) in game_state.entity_iterator() {
        let col = if &Some(*eid) == controlling {
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
