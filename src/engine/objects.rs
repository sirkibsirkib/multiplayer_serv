use std::collections::HashMap;
use ::identity::{ObjectID,AssetID};

#[derive(Serialize,Deserialize,Debug)]
pub struct ObjectDataSet {
    map : HashMap<ObjectID,ObjectData>,
}

impl ObjectDataSet {
    pub const SAVE_PATH : &'static str = "./object_data_set.lel";
    pub fn new() -> ObjectDataSet {
        ObjectDataSet {map : HashMap::new()}
    }

    pub fn get(&self, oid : ObjectID) -> Option<&ObjectData> {
        self.map.get(&oid)
    }

    pub fn insert(&mut self, oid : ObjectID, data : ObjectData) {
        self.map.insert(oid, data);
    }
}

#[derive(Serialize,Deserialize,Copy,Clone,Debug)]
pub struct ObjectData {
    pub aid : AssetID,
    pub width_meters : f64,
}

impl ObjectData {
    pub fn new(aid : AssetID, width_meters : f64) -> ObjectData {
        ObjectData {
            aid : aid,
            width_meters : width_meters,
        }
    }
}
