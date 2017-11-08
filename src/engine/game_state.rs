use std::collections::HashMap;

pub const UPDATES_PER_SEC : u64 = 32;
pub type EntityID = u64;
pub type LocationID = u32;

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub struct Point {
    pub x : f64,
    pub y : f64,
}

impl Point {
    pub fn new(x : f64, y : f64) -> Point {
        Point {
            x : x,
            y : y,
        }
    }
    // pub const NULL: Point = Point{x:0.0, y:0.0};
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Location {
    entities : HashMap<EntityID, Entity>,
}

impl Location {
    pub fn new() -> Location {
        Location {
            entities : HashMap::new(),
        }
    }

    pub fn start_location() -> Location {
        Location {
            entities : HashMap::new(),
        }
    }

    pub fn filename(lid : LocationID) -> String {
        format!("loc_{}", lid)
    }



    pub fn contains_entity(&self, eid : EntityID) -> bool {
        self.entities.contains_key(&eid)
    }

    pub fn place_inside(&mut self, eid : EntityID, e : Entity) {
        self.entities.insert(eid, e);
    }

    // pub fn add_entity(&mut self, id : EntityID, e : Entity) {
    //     self.entities.insert(id, e);
    // }

    pub fn entity_move_to(&mut self, id : EntityID, pt : Point) {
        //TODO count synch errors. when you pass a threshold you trigger a RESYNCH
        if let Some(x) = self.entities.get_mut(& id) {
            x.move_to(pt);
        }
    }

    pub fn entity_iterator<'a>(&'a self) -> Box<Iterator<Item=(&EntityID,&Entity)> + 'a> {
        Box::new(
            self.entities.iter()
        )
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Entity {
    p : Point,
}

impl Entity {
    pub fn new(p : Point) -> Entity {
        Entity {
            p : p,
        }
    }
    pub fn p(&self) -> &Point {
        &self.p
    }

    #[inline]
    pub fn move_to(&mut self, p : Point) {
        self.p = p;
    }
}
