// use std::collections::HashMap;
use bidir_map::BidirMap;
use super::Point;
use std::collections::{HashSet,HashMap};
use ::network::messaging::Diff;

use ::identity::*;
use ::engine::procedural::{NoiseField};
use ::identity::{SuperSeed,ObjectID};

#[derive(Serialize,Deserialize,Debug,Copy,Clone)]
pub struct LocationPrimitive {
    pub cells_wide : u16,
    pub cells_high : u16,
    pub cell_to_meters : f64,
    pub super_seed : SuperSeed,
}

#[derive(Debug)]
pub struct Location {
    location_primitive : LocationPrimitive,
    entities : BidirMap<EntityID, Point>,
    objects : HashMap<ObjectID,HashSet<Point>>,
    nfield_height : NoiseField,
}

fn generate_objects(nf : &NoiseField, loc_prim : &LocationPrimitive) -> HashMap<ObjectID,HashSet<Point>> {
    let mut v = HashMap::new();
    let mut zero_set = HashSet::new();
    for i in 0..loc_prim.cells_wide {
        for j in 0..loc_prim.cells_high {
            let pt = [i as i16, j as i16];
            if i == j || i == 0 || j == 0 || i == loc_prim.cells_wide-1 || j == loc_prim.cells_high-1 {
                 zero_set.insert(pt);
            }
            // if nf.sample(pt) != -90.3 { //DEBUG
            //     zero_set.insert(pt);
            // }
        }
    }
    v.insert(0, zero_set);
    v
}

impl Location {
    pub fn get_location_primitive(&self) -> &LocationPrimitive {
        &self.location_primitive
    }

    pub fn point_is_free(&self, pt : Point) -> bool {
        if self.entities.get_by_second(&pt).is_some() {
            return false;
        }
        for pt_set in self.objects.values() {
            for p in pt_set.iter() {
                if *p == pt {
                    return true;
                }
            }
        }
        return false;
    }

    pub fn new(location_primitive : LocationPrimitive) -> Location {
        let nf = NoiseField::from_super_seed(location_primitive.super_seed);
        let objects = generate_objects(&nf, &location_primitive);
        Location {
            location_primitive : location_primitive,
            entities : BidirMap::new(),
            objects : objects,
            nfield_height : nf,
        }
    }

    pub fn free_point(&self) -> Option<Point> {
        for i in 0..self.location_primitive.cells_wide as i16 {
            for j in 0..self.location_primitive.cells_high as i16 {
                let p : Point = [i,j];
                if self.point_is_free(p) {
                    return Some(p)
                }
            }
        }
        None
    }

    fn remove_eid(&mut self, eid : EntityID) -> Option<Point> {
        if let Some((_,pt)) = self.entities.remove_by_first(&eid) {
            Some(pt)
        } else {
            None
        }
    }

    pub fn point_of(&self, eid : EntityID) -> Option<Point> {
        if let Some(pt) = self.entities.get_by_first(&eid) {
            Some(*pt)
        } else {
            None
        }
    }

    fn entity_at(&self, pt : Point) -> Option<EntityID> {
        if let Some(x) = self.entities.get_by_second(&pt) {
            Some(*x)
        } else {
            None
        }
    }

    pub fn apply_diff(&mut self, diff : Diff) -> Result<(),()> {
        match diff {
            Diff::MoveEntityTo(eid,pt) => {
                if let Some(old_pt) = self.remove_eid(eid) {
                    if old_pt == pt {
                        self.entities.insert(eid, pt);
                        Err(())
                    } else {
                        self.entities.insert(eid, pt);
                        Ok(())
                    }
                } else {
                    Err(())
                }
            },
            Diff::PlaceInside(eid,pt) => {
                if self.entities.get_by_first(&eid) == None
                &&  self.entities.get_by_second(&pt) == None {
                    self.entities.insert(eid, pt);
                    Ok(())
                } else {
                    Err(())
                }
            }
        }
    }

    pub fn oid_points(&self, oid : ObjectID) -> Option<&HashSet<Point>> {
        self.objects.get(&oid)
    }

    pub fn oid_iterator<'a>(&'a self) -> Box<Iterator<Item=ObjectID> + 'a> {
        Box::new(
            self.objects.keys().map(|x| *x)
        )
    }

    pub fn object_iterator<'a>(&'a self) -> Box<Iterator<Item=(&ObjectID,&HashSet<Point>)> + 'a> {
        Box::new(
            self.objects.iter()
        )
    }

    pub fn entity_iterator<'a>(&'a self) -> Box<Iterator<Item=(&EntityID,&Point)> + 'a> {
        Box::new(
            self.entities.iter()
        )
    }
}
