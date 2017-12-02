use ::identity::*;
use network::messaging::MsgToClient;
use network::messaging::MsgToServer;
use network::ProtectedQueue;
use std::time::{Instant,Duration};
use std::collections::{HashMap,HashSet};
use ::engine::game_state::locations::{Location,LocationPrimitive};
use ::engine::game_state::worlds::{World,WorldPrimitive};
use saving::SaverLoader;
use engine::objects::*;
use engine::entities::*;
use utils::traits::*;
use std::hash::Hash;
use std::sync::{Arc,Mutex};

// struct ThingManager<K,V>
// where
// K : KnowsSaveSuffix + Hash + Eq,
// V : KnowsSavePrefix {
// 	in_memory: HashMap<K,V>,
// }

// impl<K,V> ThingManager<K,V>
// where
// K : KnowsSaveSuffix + Hash + Eq,
// V : KnowsSavePrefix {
// 	pub fn new() -> ThingManager<K,V> {
// 		ThingManager {
// 			in_memory: HashMap::new(),
// 		}
// 	}

// 	pub fn fast_populate()
// }

#[derive(Debug)]
struct ToAcquire {
    locations: HashSet<LocationID>,
    location_prims: HashSet<LocationID>,
    worlds: HashSet<WorldID>,
    world_prims: HashSet<WorldID>,
    objects: HashSet<ObjectID>,
    entities: HashSet<EntityID>,
}

impl ToAcquire {
    fn new() -> Self {
        ToAcquire {
            locations: HashSet::new(),
            location_prims: HashSet::new(),
            worlds: HashSet::new(),
            world_prims: HashSet::new(),
            objects: HashSet::new(),
            entities: HashSet::new(),
        }
    }
}


#[derive(Debug)]
pub struct ClientResources {
	locations: HashMap<LocationID, Location>,
	location_prims: HashMap<LocationID, LocationPrimitive>,
	worlds: HashMap<WorldID, World>,
	world_prims: HashMap<WorldID, WorldPrimitive>,
	objects: HashMap<ObjectID, ObjectData>,
	entities: HashMap<EntityID, EntityData>,

	to_acquire: Arc<Mutex<ToAcquire>>,

	last_req_at: Instant,
	req_pause_time: Duration,
	client_out : Arc<ProtectedQueue<MsgToServer>>,
	sl: SaverLoader,
}

impl ClientResources {
	pub fn new(sl: SaverLoader, client_out: Arc<ProtectedQueue<MsgToServer>>, req_pause_time: Duration) -> ClientResources {
		ClientResources {
			locations: HashMap::new(),
			location_prims: HashMap::new(),
			worlds: HashMap::new(),
			world_prims: HashMap::new(),
			objects: HashMap::new(),
			entities: HashMap::new(),
			last_req_at: Instant::now(),
			req_pause_time: req_pause_time,
            to_acquire: Arc::new(Mutex::new(ToAcquire::new())),
			client_out : client_out,
			sl: sl,
		}
	}

	/*
	on populate call, try the following in order:
	1. hashmap hit (loaded and in memory)
	2. file hit (locally stored)
	3. <recursive populate call for dependent type, eg World->WorldPrimitive>
	   if recursive call hit: derive new object
	4. Failed! Write a request if req_pause_time old enough, return Err
	*/

	fn fast_world_prim_populate(&mut self, wid: WorldID) -> bool {
		if self.world_prims.contains_key(&wid) {
			//.1
			true
		} else if let Ok(wp) = self.sl.load_with_key::<WorldPrimitive,WorldID>(wid) {
			//.2
			self.world_prims.insert(wid, wp);
			true
			//No .3 possible! no dependent type
		} else {
			//.4
			let now = Instant::now();
			if self.last_req_at + self.req_pause_time < now {
				self.client_out.lock_push_notify (
					MsgToServer::RequestWorldData(wid)
				)
			}
			false
		}
	}

	fn fast_world_populate(&mut self, wid: WorldID) -> bool {
		if self.worlds.contains_key(&wid) {
			//.1
			true
		// } else if let Ok(w) = self.sl.load::<World,WorldID>(wid) {
			// .2
			// self.world.insert(wid, w);
			// true
			//No loading for world
		} else if self.fast_world_prim_populate(wid) {
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

	fn fast_location_prim_populate(&mut self, lid: LocationID) -> bool {
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

	fn fast_location_populate(&mut self, lid: LocationID) -> bool {
		if self.worlds.contains_key(&lid) {
			//.1
			true
			//.2 not possible. worlds can't be loaded
		}  else if self.fast_location_prim_populate(lid) {
			//.3
			let lp : LocationPrimitive = self.location_prims.get(&lid).expect("you said..").clone();
			let wid = lp.wid;
			if self.fast_world_populate(wid) {
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

	pub fn fast_object_populate(&mut self, oid: ObjectID) -> bool {
		if self.objects.contains_key(&oid) {
			//.1
			true
		} else if let Ok(od) = self.sl.load_with_key::<ObjectData,ObjectID>(oid) {
			self.objects.insert(oid, od);
			true
		} else {
			let now = Instant::now();
			if self.last_req_at + self.req_pause_time < now {
				self.client_out.lock_push_notify (
					MsgToServer::RequestObjectData(oid)
				);
			}
			false
		}
	}

	pub fn fast_entity_populate(&mut self, eid: EntityID) -> bool {
		if self.entities.contains_key(&eid) {
			//.1
			true
		} else if let Ok(ed) = self.sl.load_with_key::<EntityData,EntityID>(eid) {
			self.entities.insert(eid, ed);
			true
		} else {
			let now = Instant::now();
			if self.last_req_at + self.req_pause_time < now {
				self.client_out.lock_push_notify (
					MsgToServer::RequestEntityData(eid)
				);
			}
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
			MsgToClient::GiveObjectData(oid, od) => {
				self.objects.insert(oid,od);
			},
			MsgToClient::GiveEntityData(eid, ed) => {
				self.entities.insert(eid,ed);
			},
			m => {
				println!("Client resources got unexpected msg! {:?}", m);
			},
		}
	}

	// pub fn get_world_primitive(&mut self, wid: WorldID) -> Result<&WorldPrimitive,()> {
	// 	if self.fast_world_prim_populate(wid) {
	// 		Ok(self.world_prims.get(&wid).expect("kkfam"))
	// 	} else {
	// 		Err(())
	// 	}
	// }

	// pub fn get_world(&mut self, wid: WorldID) -> Result<&World,()> {
	// 	if self.fast_world_populate(wid) {
	// 		Ok(self.worlds.get(&wid).expect("kkfam"))
	// 	} else {
	// 		Err(())
	// 	}
	// }

	// pub fn get_location_primitive(&mut self, lid: LocationID) -> Result<&LocationPrimitive,()> {
	// 	if self.fast_location_prim_populate(lid) {
	// 		Ok(self.location_prims.get(&lid).expect("kkfam"))
	// 	} else {
	// 		Err(())
	// 	}
	// }

	// pub fn get_location(&mut self, lid: LocationID) -> Result<&Location,()> {
	// 	if self.fast_location_populate(lid) {
	// 		Ok(self.locations.get(&lid).expect("kkfam"))
	// 	} else {
	// 		Err(())
	// 	}
	// }

	// pub fn get_object(&mut self, oid: ObjectID) -> Result<&ObjectData,()> {
	// 	if self.fast_object_populate(oid) {
	// 		Ok(self.objects.get(&oid).unwrap())
	// 	} else {
	// 		Err(())
	// 	}
	// }

	// pub fn get_entity(&mut self, eid: EntityID) -> Result<&EntityData,()> {
	// 	if self.fast_entity_populate(eid) {
	// 		Ok(self.entities.get(&eid).unwrap())
	// 	} else {
	// 		Err(())
	// 	}
	// }

	pub fn try_get_world_prim(&self, wid: WorldID) -> Option<&WorldPrimitive> {
        if let Some(x) = self.world_prims.get(&wid) {
            Some(x)
        } else {
            self.to_acquire.lock().unwrap().world_prims.insert(wid);
            None
        }
    }

    pub fn try_get_world(&self, wid: WorldID) -> Option<&World> {
        if let Some(x) = self.worlds.get(&wid) {
            Some(x)
        } else {
            self.to_acquire.lock().unwrap().worlds.insert(wid);
            None
        }
    }

    pub fn try_get_location_prim(&self, lid: LocationID) -> Option<&LocationPrimitive> {
        if let Some(x) = self.location_prims.get(&lid) {
            Some(x)
        } else {
            self.to_acquire.lock().unwrap().location_prims.insert(lid);
            None
        }
    }

    pub fn try_get_location(&self, lid: LocationID) -> Option<&Location> {
        if let Some(x) = self.locations.get(&lid) {
            Some(x)
        } else {
            self.to_acquire.lock().unwrap().locations.insert(lid);
            None
        }
    }

    pub fn try_get_object(&self, oid: ObjectID) -> Option<&ObjectData> {
        if let Some(x) = self.objects.get(&oid) {
            Some(x)
        } else {
            self.to_acquire.lock().unwrap().objects.insert(oid);
            None
        }
    }

    pub fn try_get_entity(&self, eid: EntityID) -> Option<&EntityData> {
        if let Some(x) = self.entities.get(&eid) {
            Some(x)
        } else {
            self.to_acquire.lock().unwrap().entities.insert(eid);
            None
        }
    }

	///////////////////////////////////////////////////////////////////////

	pub fn get_mut_world_primitive(&mut self, wid: WorldID) -> Result<&mut WorldPrimitive,()> {
		if self.fast_world_prim_populate(wid) {
			Ok(self.world_prims.get_mut(&wid).expect("kkfam"))
		} else {
			Err(())
		}
	}

	pub fn get_mut_world(&mut self, wid: WorldID) -> Result<&mut World,()> {
		if self.fast_world_populate(wid) {
			Ok(self.worlds.get_mut(&wid).expect("kkfam"))
		} else {
			Err(())
		}
	}

	pub fn get_mut_location_primitive(&mut self, lid: LocationID) -> Result<&mut LocationPrimitive,()> {
		if self.fast_location_prim_populate(lid) {
			Ok(self.location_prims.get_mut(&lid).expect("kkfam"))
		} else {
			Err(())
		}
	}

	pub fn get_mut_location(&mut self, lid: LocationID) -> Result<&mut Location,()> {
		if self.fast_location_populate(lid) {
			Ok(self.locations.get_mut(&lid).expect("kkfam"))
		} else {
			Err(())
		}
	}

	pub fn perform_acquisitions(&mut self) {
        // Call periodically to acquire things that have been requested but couldn't be returned
        // let mut t_o = ;
        //TODO
        {
        	let v : Vec<WorldID>;
        	{
        		let mut t = self.to_acquire.lock().unwrap();
        		v = t.world_prims.drain().collect();
        	} 
        	for wid in v {
	            self.fast_world_prim_populate(wid);
	        }
        }
        // for wid in t_o.worlds.drain() {
        //     self.fast_world_populate(wid);
        // }
        // for lid in t_o.location_prims.drain() {
        //     self.fast_location_prim_populate(lid);
        // }
        // for lid in t_o.locations.drain() {
        //     self.fast_location_populate(lid);
        // }
    }
}