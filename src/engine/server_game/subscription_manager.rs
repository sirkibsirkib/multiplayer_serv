use identity::*;
use std::collections::HashMap;

pub struct SubscriptionManager {
	subs: HashMap<LocationID,ClientIDSet>,
}

impl SubscriptionManager {
	pub fn new() -> SubscriptionManager{subs::HashMap::new()}

	pub fn subscribe(&mut self, lid: LocationID, cid: ClientID) {
		if self.subs.contains_key(&lid) {
			let mut s = self.subs.get_mut(&lid);
			s.set(cid, true);
		} else {
			self.subs.insert(lid, ClientIDSet::new_just_one(cid));
		}
	}

	pub unsubscribe(&mut self, lid: LocationID, cid: ClientID) {
		if self.subs.contains_key(&lid) {
			let mut s = self.subs.get_mut(&lid);
			s.set(cid, false);
			if s.is_empty() {
				self.subs.remove(&lid);
			}
		}
	}

	pub fn iter_subs_for(&self, lid: LocationID) -> ClientIDSetIntoIterator {
		self.subs.iter_set_pos()
	}

	pub fn get_subs_for(&self, lid: LocationID) -> ClientIDSet {
		self.subs.clone()
	}
}