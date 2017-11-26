use ::identity::*;
use rand::{Rng,Isaac64Rng};
use std::collections::HashMap;
use ::engine::game_state::locations::{Location,LocationPrimitive};
use ::engine::game_state::worlds::{World,WorldPrimitive};
use saving::SaverLoader;
use utils::traits::*;
use ::identity::UniquePoint;

#[derive(Debug,Serialize,Deserialize)]
struct Portals {
    portals: HashMap<UniquePoint,UniquePoint>,
}

impl KnowsSavePrefix for Portals {
    fn get_save_prefix() -> String {
         "portals".to_owned()
    }
}


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
            return
        }
        self.world_prim_populate(wid);
        let wp = self.world_prims.get(&wid).expect("dawg");
        let w = World::new(wp.clone());
        self.worlds.insert(wid, w);
    }

    fn location_prim_populate(&mut self, lid: LocationID) {
        if self.location_prims.contains_key(&lid) {
            //.1
            return
        } else if let Ok(lp) = self.sl.load_with_key::<LocationPrimitive,LocationID>(lid) {
            //.2
            self.location_prims.insert(lid, lp);
            return
        }
        panic!("Unknown LocPrim creation requested!");
    }

    fn location_populate(&mut self, lid: LocationID) {
        if self.worlds.contains_key(&lid) {
            //.1
            return
        }
        self.location_prim_populate(lid);
        let lp : LocationPrimitive = self.location_prims.get(&lid).expect("you said..").clone();
        self.world_populate(lp.wid);
        let w = self.worlds.get(&lp.wid).expect("you said..");
        let world_zone = w.get_zone(lp.zone_id);
        let l = Location::generate_new(lp.clone(), world_zone.clone());
        self.locations.insert(lid, l);
    }

    ///////////////////////////// PUBLIC ///////////////////////

    pub fn get_world_primitive(&mut self, wid: WorldID) -> &WorldPrimitive {
        self.world_prim_populate(wid);
        self.world_prims.get(&wid).expect("kkfam")
    }

    pub fn get_world(&mut self, wid: WorldID) -> &World {
        self.world_populate(wid);
        self.worlds.get(&wid).expect("kkfam")
    }

    pub fn get_location_primitive(&mut self, lid: LocationID) -> &LocationPrimitive {
        self.location_prim_populate(lid);
        self.location_prims.get(&lid).expect("kkfam")
    }

    pub fn get_location(&mut self, lid: LocationID) -> &Location {
        self.location_populate(lid);
        self.locations.get(&lid).expect("kkfam")
    }

    pub fn get_mut_world_primitive(&mut self, wid: WorldID) -> &mut WorldPrimitive {
        self.world_prim_populate(wid);
        self.world_prims.get_mut(&wid).expect("kkfam")
    }

    pub fn get_mut_world(&mut self, wid: WorldID) -> &mut World {
        self.world_populate(wid);
        self.worlds.get_mut(&wid).expect("kkfam")
    }

    pub fn get_mut_location_primitive(&mut self, lid: LocationID) -> &mut LocationPrimitive {
        self.location_prim_populate(lid);
        self.location_prims.get_mut(&lid).expect("kkfam")
    }

    pub fn get_mut_location(&mut self, lid: LocationID) -> &mut Location {
        self.location_populate(lid);
        self.locations.get_mut(&lid).expect("kkfam")
    }

    pub fn save_all(&mut self) {
        for (lid,lp) in self.location_prims.iter() {
            self.sl.save_with_key(lp, *lid);
        }
        for (wid,wp) in self.world_prims.iter() {
            self.sl.save_with_key(wp, *wid);
        }
    }

    pub fn unload_lid(&mut self, lid: LocationID) {
        if let Some(lp) = self.location_prims.remove(&lid) {
            self.sl.save_with_key(&lp, lid);
        }
        let _ = self.locations.remove(&lid);
    }

    pub fn unload_wid(&mut self, wid: WorldID) {

    }
}