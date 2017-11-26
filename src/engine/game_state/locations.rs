// use std::collections::HashMap;
use bidir_map::BidirMap;
// use super::Point;
use ::points::DPoint2;
use std::collections::{HashSet,HashMap};
use ::network::messaging::Diff;
use ::rand::{SeedableRng,Isaac64Rng};
use super::worlds::zones::Zone;

use ::identity::*;
use ::utils::noise::*;
use ::utils::traits::*;
use ::identity::{SuperSeed,ObjectID};
// use super::worlds::zones::Zone;
use super::worlds::START_WORLD;


lazy_static! {
    pub static ref START_LOC_PRIM : LocationPrimitive = LocationPrimitive::new(0, 0, 1.0, 0);
    pub static ref START_LOC : Location = Location::generate_new(*START_LOC_PRIM, START_WORLD.get_zone(0).clone());
}



#[derive(Serialize,Deserialize,Debug,Copy,Clone)]
pub struct LocationPrimitive {
    // pub world_zone: Zone,
    pub wid: WorldID,
    pub zone_id: usize,
    // pub cells_wide : u16,
    // pub cells_high : u16,
    pub cell_to_meters : f64,
    pub super_seed : SuperSeed,
}

impl KnowsSavePrefix for LocationPrimitive {
    fn get_save_prefix() -> String {
        "loc_prim".to_owned()
    }
}

impl LocationPrimitive {
    pub fn save_path(lid: LocationID) -> String {
        format!("loc_prim_{}", lid)
    }
    pub fn new(wid: WorldID, zone_id:usize, cell_to_meters : f64, super_seed: u64) -> LocationPrimitive {
        LocationPrimitive{wid:wid, zone_id:zone_id, cell_to_meters:cell_to_meters, super_seed:super_seed}
    }
}


#[derive(Debug)]
pub struct Location {
    world_zone: Zone,
    location_primitive : LocationPrimitive,
    entities : BidirMap<EntityID, DPoint2>,
    objects : HashMap<ObjectID,HashSet<DPoint2>>,
    nfield_height : NoiseField,
}

impl AppliesDiff<Diff> for Location {
    fn apply_diff(&mut self, diff: &Diff) {

    }
}

fn generate_objects(nf : &NoiseField, loc_prim : &LocationPrimitive, cells_wide: i32, cells_high: i32,) -> HashMap<ObjectID,HashSet<DPoint2>> {
    let mut v = HashMap::new();
    let mut zero_set = HashSet::new();
    for i in 0..cells_wide {
        for j in 0..cells_high {
            let pt : DPoint2 = DPoint2::new(i, j);
            if nf.sample_2d(pt.continuous().scale(0.2)) > 0.01 {
                 zero_set.insert(pt);
            }
        }
    }
    v.insert(0, zero_set);
    v
}

impl Location {
    pub fn generate_new(lp: LocationPrimitive, world_zone: Zone) -> Location {
        let mut rng = Isaac64Rng::from_seed(&[lp.super_seed]);
        let nf = NoiseField::generate(&mut rng, [0.2, 1.0], 2);
        let objects = generate_objects(&nf, &lp, world_zone.get_samples_per_row(), world_zone.get_samples_per_col());
        Location {
            world_zone: world_zone,
            location_primitive : lp,
            entities : BidirMap::new(),
            objects : objects,
            nfield_height : nf,
        }
    }

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

    pub fn cells_wide(&self) -> i32 {
        self.world_zone.get_samples_per_row()
    }

    pub fn cells_high(&self) -> i32 {
        self.world_zone.get_samples_per_col()
    }

    pub fn free_point(&self) -> Option<DPoint2> {
        for i in 0..self.cells_wide() {
            for j in 0..self.cells_high() {
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
