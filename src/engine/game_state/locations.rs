// use std::collections::HashMap;
use bidir_map::BidirMap;
// use super::Point;
use ::points::DPoint2;
use std::collections::{HashSet,HashMap};
use ::network::messaging::Diff;
use ::rand::{SeedableRng,Rng,Isaac64Rng};

use ::identity::*;
use ::engine::procedural::{NoiseField};
use ::identity::{SuperSeed,ObjectID};

use ::engine::primitives::*;

impl Primitive<Location> for LocationPrimitive {
    fn generate_new(self) -> Location {
        let mut rng = Isaac64Rng::from_seed(&[self.super_seed]);
        let nf = NoiseField::generate(&mut rng, [0.2, 1.0], 2);
        let objects = generate_objects(&nf, &self);
        Location {
            location_primitive : self,
            entities : BidirMap::new(),
            objects : objects,
            nfield_height : nf,
        }
    }
}

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
    entities : BidirMap<EntityID, DPoint2>,
    objects : HashMap<ObjectID,HashSet<DPoint2>>,
    nfield_height : NoiseField,
}

impl AppliesDiff<Diff> for Location {
    fn apply_diff(&mut self, diff: &Diff) {

    }
}

fn generate_objects(nf : &NoiseField, loc_prim : &LocationPrimitive) -> HashMap<ObjectID,HashSet<DPoint2>> {
    let mut v = HashMap::new();
    let mut zero_set = HashSet::new();
    for i in 0..loc_prim.cells_wide {
        for j in 0..loc_prim.cells_high {
            let pt : DPoint2 = DPoint2::new(i as i32, j as i32);
            if nf.sample_2d(pt.continuous().scale(0.2)) > 0.01 {
                 zero_set.insert(pt);
            }
        }
    }
    v.insert(0, zero_set);
    v
}

impl Location {
    pub fn get_location_primitive(&self) -> &LocationPrimitive {
        &self.location_primitive
    }

    pub fn point_is_free(&self, pt : DPoint2) -> bool {
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

    pub fn free_point(&self) -> Option<DPoint2> {
        for i in 0..self.location_primitive.cells_wide as i32 {
            for j in 0..self.location_primitive.cells_high as i32 {
                let p : DPoint2 = DPoint2::new(i,j);
                if self.point_is_free(p) {
                    return Some(p)
                }
            }
        }
        None
    }

    fn remove_eid(&mut self, eid : EntityID) -> Option<DPoint2> {
        self.entities.remove_by_first(&eid)
        .map(|eid_pt| eid_pt.1)
    }

    pub fn point_of(&self, eid : EntityID) -> Option<DPoint2> {
        self.entities.get_by_first(&eid)
        .map(|pt| *pt)
    }

    fn entity_at(&self, pt : DPoint2) -> Option<EntityID> {
        self.entities.get_by_second(&pt)
        .map(|ent| *ent)
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

    pub fn oid_points(&self, oid : ObjectID) -> Option<&HashSet<DPoint2>> {
        self.objects.get(&oid)
    }

    pub fn oid_iterator<'a>(&'a self) -> Box<Iterator<Item=ObjectID> + 'a> {
        Box::new(
            self.objects.keys().map(|x| *x)
        )
    }

    pub fn object_iterator<'a>(&'a self) -> Box<Iterator<Item=(&ObjectID,&HashSet<DPoint2>)> + 'a> {
        Box::new(
            self.objects.iter()
        )
    }

    pub fn entity_iterator<'a>(&'a self) -> Box<Iterator<Item=(&EntityID,&DPoint2)> + 'a> {
        Box::new(
            self.entities.iter()
        )
    }
}
