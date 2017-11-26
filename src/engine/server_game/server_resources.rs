use ::identity::*;
use network::messaging::MsgToClient;
use rand::{Rng,Isaac64Rng};
use std::sync::Arc;
use network::messaging::MsgToServer;
use network::ProtectedQueue;
use std::time::{Instant,Duration};
use std::collections::HashMap;
use ::engine::game_state::locations::{Location,LocationPrimitive};
use ::engine::game_state::worlds::{World,WorldPrimitive};
use saving::SaverLoader;


#[derive(Debug)]
pub struct ServerResources {
    locations: HashMap<LocationID, Location>,
    location_prims: HashMap<LocationID, LocationPrimitive>,
    worlds: HashMap<WorldID, World>,
    world_prims: HashMap<WorldID, WorldPrimitive>,
    sl: SaverLoader,
    rng: Isaac64Rng,
}

impl ServerResources {
    pub fn new(sl: SaverLoader, rng: Isaac64Rng) -> ServerResources {
        ServerResources {
            locations: HashMap::new(),
            location_prims: HashMap::new(),
            worlds: HashMap::new(),
            world_prims: HashMap::new(),
            rng: rng,
            sl: sl,
        }
    }

    /*
    1. hashmap load
    2. file load
    3. recursive ASSURE call
       derive, save, return
    */

    fn world_prim_populate(&mut self, wid: WorldID) {
        if self.world_prims.contains_key(&wid) {
            //.1
            return
        } 
        if let Ok(wp) = self.sl.load_with_key::<WorldPrimitive,WorldID>(wid) {
            //.2
            self.world_prims.insert(wid, wp);
            return
        }
        //make new!
        let wp = WorldPrimitive::new(self.rng.gen(), self.rng.gen());
        self.world_prims.insert(wid, wp);
    }    

    fn world_populate(&mut self, wid: WorldID) {
        if self.worlds.contains_key(&wid) {
            //.1
            true
        // } else if let Ok(w) = self.sl.load::<World,WorldID>(wid) {
            // .2
            // self.world.insert(wid, w);
            // true
            //No loading for world
        } else if self.world_prim_populate(wid) {
            //.3
            let wp = self.world_prims.get(&wid).expect("you said..");
            let w = World::new(wp.clone());
            //cache locally
            // let _ = self.sl.save::<World,WorldID>(&w, wid); //TODO can't save worlds?
            self.worlds.insert(wid, w);
            true
        } else {
            //.4
            //.3 already sent a WorldPrim request
            false
        }
    }

    fn location_prim_populate(&mut self, lid: LocationID) {
        if self.location_prims.contains_key(&lid) {
            //.1
            true
        } else if let Ok(lp) = self.sl.load_with_key::<LocationPrimitive,LocationID>(lid) {
            //.2
            self.location_prims.insert(lid, lp);
            true
        } else {
            //No .3 possible! no dependent type
            //.4
            let now = Instant::now();
            if self.last_req_at + self.req_pause_time < now {
                self.client_out.lock_push_notify (
                    MsgToServer::RequestLocationData(lid)
                )
            }
            false
        }
    }

    fn location_populate(&mut self, lid: LocationID) {
        if self.worlds.contains_key(&lid) {
            //.1
            true
            //.2 not possible. worlds can't be loaded
        }  else if self.location_prim_populate(lid) {
            //.3
            let lp : LocationPrimitive = self.location_prims.get(&lid).expect("you said..").clone();
            let wid = lp.wid;
            if self.world_populate(wid) {
                let w = self.worlds.get(&wid).expect("you said..");
                let world_zone = w.get_zone(lp.zone_id);
                let l = Location::generate_new(lp.clone(), world_zone.clone());
                self.locations.insert(lid, l);
                true
            } else {
                false
            }
        } else {
            //.4
            //.3 already sent all requests
            false
        }
    }

    ///////////////////////////// PUBLIC ///////////////////////

    pub fn server_sent_data(&mut self, msg: MsgToClient) {
        match msg {
            MsgToClient::GiveLocationPrimitive(lid, lp) => {
                self.location_prims.insert(lid, lp);
            },
            MsgToClient::GiveWorldPrimitive(wid, wp) => {
                self.world_prims.insert(wid,wp);
            },
            m => {
                println!("Client resources got unexpected msg! {:?}", m);
            },
        }
    }

    pub fn get_world_primitive(&mut self, wid: WorldID) -> Result<&WorldPrimitive,()> {
        if self.world_prim_populate(wid) {
            Ok(self.world_prims.get(&wid).expect("kkfam"))
        } else {
            Err(())
        }
    }

    pub fn get_world(&mut self, wid: WorldID) -> Result<&World,()> {
        if self.world_populate(wid) {
            Ok(self.worlds.get(&wid).expect("kkfam"))
        } else {
            Err(())
        }
    }

    pub fn get_location_primitive(&mut self, lid: LocationID) -> Result<&LocationPrimitive,()> {
        if self.location_prim_populate(lid) {
            Ok(self.location_prims.get(&lid).expect("kkfam"))
        } else {
            Err(())
        }
    }

    pub fn get_location(&mut self, lid: LocationID) -> Result<&Location,()> {
        if self.location_populate(lid) {
            Ok(self.locations.get(&lid).expect("kkfam"))
        } else {
            Err(())
        }
    }

    ///////////////////////////////////////////////////////////////////////

    pub fn get_mut_world_primitive(&mut self, wid: WorldID) -> Result<&mut WorldPrimitive,()> {
        if self.world_prim_populate(wid) {
            Ok(self.world_prims.get_mut(&wid).expect("kkfam"))
        } else {
            Err(())
        }
    }

    pub fn get_mut_world(&mut self, wid: WorldID) -> Result<&mut World,()> {
        if self.world_populate(wid) {
            Ok(self.worlds.get_mut(&wid).expect("kkfam"))
        } else {
            Err(())
        }
    }

    pub fn get_mut_location_primitive(&mut self, lid: LocationID) -> Result<&mut LocationPrimitive,()> {
        if self.location_prim_populate(lid) {
            Ok(self.location_prims.get_mut(&lid).expect("kkfam"))
        } else {
            Err(())
        }
    }

    pub fn get_mut_location(&mut self, lid: LocationID) -> Result<&mut Location,()> {
        if self.location_populate(lid) {
            Ok(self.locations.get_mut(&lid).expect("kkfam"))
        } else {
            Err(())
        }
    }
}