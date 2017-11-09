use std::collections::HashMap;
use bidir_map::BidirMap;
use super::Diff;

use super::super::identity::*;
pub const UPDATES_PER_SEC : u64 = 32;

pub type Point = [i16;2];

// #[derive(Clone, Debug, Copy, Serialize, Deserialize)]
// pub struct Point {
//     pub x : f64,
//     pub y : f64,
// }
//
// impl Point {
//     pub fn new(x : f64, y : f64) -> Point {
//         Point {
//             x : x,
//             y : y,
//         }
//     }
//     // pub const NULL: Point = Point{x:0.0, y:0.0};
// }

#[derive(Serialize,Deserialize,Debug,Copy,Clone)]
pub struct LocationPrimitive {
    pub cells_wide : u16,
    pub cells_high : u16,
    pub cell_width : f32, //meters
}

pub struct Location {
    location_primitive : LocationPrimitive,
    entities : BidirMap<EntityID, Point>,
}

impl Location {
    pub fn get_location_primitive(&self) -> &LocationPrimitive {
        &self.location_primitive
    }

    pub fn new(location_primitive : LocationPrimitive) -> Location {
        Location {
            location_primitive : location_primitive,
            entities : BidirMap::new(),
        }
    }

    pub fn point_of(&self, eid : EntityID) -> Option<Point> {
        if let Some(pt) = self.entities.get_by_first(&eid) {
            Some(*pt)
        } else {
            None
        }
    }

    pub fn start_location() -> Location {
        Location {
            location_primitive : LocationPrimitive {
                cells_wide : 50,
                cells_high : 50,
                cell_width : 1.0,
            },
            entities : BidirMap::new(),
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
