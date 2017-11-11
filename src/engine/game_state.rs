use std::collections::HashMap;
use bidir_map::BidirMap;
use super::Diff;

use ::identity::*;
pub const UPDATES_PER_SEC : u64 = 32;
use super::procedural::{NoiseField,NoiseMaster};
use ::identity::SuperSeed;

pub type Point = [i16;2];

#[derive(Serialize,Deserialize,Debug,Copy,Clone)]
pub struct LocationPrimitive {
    pub cells_wide : u16,
    pub cells_high : u16,
    pub cell_width : f32, //meters
    pub super_seed : SuperSeed,
}

#[derive(Debug)]
pub struct Location {
    location_primitive : LocationPrimitive,
    entities : BidirMap<EntityID, Point>,
    noise_field : NoiseField,
}

impl Location {
    pub fn get_location_primitive(&self) -> &LocationPrimitive {
        &self.location_primitive
    }

    pub fn new(location_primitive : LocationPrimitive, nm : &NoiseMaster) -> Location {
        Location {
            location_primitive : location_primitive,
            entities : BidirMap::new(),
            noise_field : NoiseField::from_super_seed(location_primitive.super_seed, nm),
        }
    }

    pub fn point_of(&self, eid : EntityID) -> Option<Point> {
        if let Some(pt) = self.entities.get_by_first(&eid) {
            Some(*pt)
        } else {
            None
        }
    }

    pub fn apply_diff(&mut self, diff : Diff) {
        match diff {
            Diff::MoveEntityTo(eid,pt) => {
                if ! self.entities.contains_first_key(&eid) {
                    panic!("Moving entity I don't have!");
                }
                self.entities.insert(eid, pt);
            },
            Diff::PlaceInside(eid,pt) => {
                self.entities.insert(eid, pt);
            }
        }
    }

    pub fn entity_iterator<'a>(&'a self) -> Box<Iterator<Item=(&EntityID,&Point)> + 'a> {
        Box::new(
            self.entities.iter()
        )
    }
}

#[derive(Debug)]
pub struct Entity {
    // p : Point,
}

impl Entity {
    pub fn new() -> Entity {
        Entity {
            // p : p,
        }
    }
}
