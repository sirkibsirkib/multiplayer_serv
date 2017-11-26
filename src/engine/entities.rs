use std::collections::HashMap;
use ::identity::{EntityID,AssetID};
use utils::traits::*;

#[derive(Serialize,Deserialize,Debug)]
pub struct EntityDataSet {
    map : HashMap<EntityID,EntityData>,
}

impl EntityDataSet {
    pub const SAVE_PATH : &'static str = "./entity_data_set.lel";
    pub fn new() -> EntityDataSet {
        EntityDataSet {map : HashMap::new()}
    }

    pub fn get(&self, eid : EntityID) -> Option<&EntityData> {
        self.map.get(&eid)
    }

    pub fn insert(&mut self, eid : EntityID, data : EntityData) {
        self.map.insert(eid, data);
    }
}

#[derive(Serialize,Deserialize,Copy,Clone,Debug)]
pub struct EntityData {
    pub aid : AssetID,
    pub width_meters : f64,
}

impl EntityData {
    pub fn new(aid : AssetID, width_meters : f64) -> EntityData {
        EntityData {
            aid : aid,
            width_meters : width_meters,
        }
    }
}


impl KnowsSavePrefix for EntityData {
    fn get_save_prefix() -> String {
        "entity".to_owned()
    }
}
