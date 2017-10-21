use std::io::Error;
use std::collections::HashMap;

pub const UPDATES_PER_SEC : u64 = 32;


pub struct GameState {
    entities : HashMap<EntityID, Entity>,
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            entities : HashMap::new(),
        }
    }

    pub fn load_from(path : &str) -> Result<GameState, &'static Error> {
        //TODO
        Ok(GameState::new())
    }

    pub fn entity_exists(&self, eid : EntityID) -> bool {
        self.entities.contains_key(&eid)
    }

    pub fn add_entity(&mut self, id : EntityID, e : Entity) {
        self.entities.insert(id, e);
    }

    pub fn entity_move_to(&mut self, id : EntityID, pt : Point) {
        //TODO count synch errors. when you pass a threshold you trigger a RESYNCH
        if let Some(x) = self.entities.get_mut(& id) {
            x.p = pt;
        }
    }

    pub fn entity_iterator<'a>(&'a self) -> Box<Iterator<Item=(&EntityID,&Entity)> + 'a> {
        Box::new(
            self.entities.iter()
        )
    }
}

pub type EntityID = u64;


#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub struct Point {
    pub x : f64,
    pub y : f64,
}

impl Point {
    // pub const NULL: Point = Point{x:0.0, y:0.0};
}

#[derive(Debug)]
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
}
