use identity::*;
use std::collections::HashMap;

pub struct SubscriptionManager {
	subs: HashMap<LocationID,ClientIDSet>,
}

impl SubscriptionManager {
	pub fn new() -> SubscriptionManager{
		SubscriptionManager {
			subs: HashMap::new(),
		}
	}

	pub fn subscribe(&mut self, lid: LocationID, cid: ClientID) {
		if self.subs.contains_key(&lid) {
			let mut s = self.subs.get_mut(&lid).unwrap();
			s.set(cid, true);
		} else {
			self.subs.insert(lid, ClientIDSet::new_just_one(cid));
		}
	}

	pub fn unsubscribe(&mut self, lid: LocationID, cid: ClientID) {
		if self.subs.contains_key(&lid) {
			let mut s = self.subs.get_mut(&lid).unwrap();
			s.set(cid, false);
			if s.is_empty() {
				self.subs.remove(&lid);
			}
		}
	}

	pub fn iter_subs_for(&self, lid: LocationID) -> ClientIDSetIntoIterator {
		match self.subs.get(&lid) {
			Some(s) => s.iter_set_pos(),
			None => ClientIDSet::new().iter_set_pos(),
		}
	}

	pub fn get_subs_for(&self, lid: LocationID) -> ClientIDSet {
		match self.subs.get(&lid) {
			Some(s) => s.clone(),
			None => ClientIDSet::new(),
		}
	}
}