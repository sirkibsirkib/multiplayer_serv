use ::identity::*;
use network::messaging::MsgToClient;
use std::sync::Arc;
use network::messaging::MsgToServer;
use network::ProtectedQueue;
use std::time::{Instant,Duration};
use std::collections::HashMap;
use ::engine::game_state::locations::{Location,LocationPrimitive};
use ::engine::game_state::worlds::{World,WorldPrimitive};
use saving::SaverLoader;


#[derive(Debug)]
pub struct ClientResources {
	locations: HashMap<LocationID, Location>,
	location_prims: HashMap<LocationID, LocationPrimitive>,
	worlds: HashMap<WorldID, World>,
	world_prims: HashMap<WorldID, WorldPrimitive>,
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
			last_req_at: Instant::now(),
			req_pause_time: req_pause_time,
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
		} else if let Ok(wp) = self.sl.load::<WorldPrimitive,WorldID>(wid) {
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
		if self.world.contains_key(&wid) {
			//.1
			true
		} else if let Ok(w) = self.sl.load::<World,WorldID>(wid) {
			//.2
			self.world.insert(wid, w);
			true
		} else if self.fast_world_prim_populate(wid) {
			//.3
			let wp = self.world_prims.get(wid).expect("you said..");
			let w = World::new(wp);
			//cache locally
			let _ = self.sl.save::<World,WorldID>(&w, wid);
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
		} else if let Ok(lp) = self.sl.load::<LocationPrimitive,LocationID>(lid) {
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
		if self.world.contains_key(&lid) {
			//.1
			true
			//.2 not possible. worlds can't be loaded
		}  else if self.fast_location_prim_populate(lid) {
			//.3
			let lp = self.location_prims.get(lid).expect("you said..");
			let wid = lp.wid;
			if self.fast_world_populate(wid) {
				let w = self.worlds.get(wid).expect("you said..");
				let world_zone = w.get_zone(lp.zone_id);
				let l = Location::generate_new(lp, world_zone);
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
		if self.fast_world_prim_populate(wid) {
			Ok(self.world_prims.get(wid).expect("kkfam"))
		} else {
			Err(())
		}
	}

	pub fn get_world(&mut self, wid: WorldID) -> Result<&World,()> {
		if self.fast_world_populate(wid) {
			Ok(self.worlds.get(wid).expect("kkfam"))
		} else {
			Err(())
		}
	}

	pub fn get_location_primitive(&mut self, lid: LocationID) -> Result<&LocationPrimitive,()> {
		if self.fast_location_prim_populate(lid) {
			Ok(self.location_prims.get(lid).expect("kkfam"))
		} else {
			Err(())
		}
	}

	pub fn get_location(&mut self, lid: LocationID) -> Result<&Location,()> {
		if self.fast_location_populate(lid) {
			Ok(self.locations.get(lid).expect("kkfam"))
		} else {
			Err(())
		}
	}

	///////////////////////////////////////////////////////////////////////

	pub fn get_mut_world_primitive(&mut self, wid: WorldID) -> Result<&mut WorldPrimitive,()> {
		if self.fast_world_prim_populate(wid) {
			Ok(self.world_prims.get_mut(wid).expect("kkfam"))
		} else {
			Err(())
		}
	}

	pub fn get_mut_world(&mut self, wid: WorldID) -> Result<&mut World,()> {
		if self.fast_world_populate(wid) {
			Ok(self.worlds.get_mut(wid).expect("kkfam"))
		} else {
			Err(())
		}
	}

	pub fn get_mut_location_primitive(&mut self, lid: LocationID) -> Result<&mut LocationPrimitive,()> {
		if self.fast_location_prim_populate(lid) {
			Ok(self.location_prims.get_mut(lid).expect("kkfam"))
		} else {
			Err(())
		}
	}

	pub fn get_mut_location(&mut self, lid: LocationID) -> Result<&mut Location,()> {
		if self.fast_location_populate(lid) {
			Ok(self.locations.get_mut(lid).expect("kkfam"))
		} else {
			Err(())
		}
	}
}