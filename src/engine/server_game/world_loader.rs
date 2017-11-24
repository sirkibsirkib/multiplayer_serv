use std::collections::HashMap;
use super::game_state::worlds::{WorldPrimitive};
use ::identity::*;
use ::noise::{Perlin,Seedable,NoiseModule};
use ::rand::{thread_rng, Rng};

#[derive(Serialize,Deserialize,Debug)]
pub struct WorldLoader {
    world_primitives : HashMap<WorldID, WorldPrimitive>,
}

impl WorldLoader {
    pub fn new() -> WorldLoader {
        WorldLoader {
            world_primitives : HashMap::new(),
        }
    }

    pub fn get_world_primitive_for(&mut self, wid: WorldID) -> WorldPrimitive {
        if ! self.world_primitives.contains_key(&wid) {
            println!("Lazily inventing a world. No biggie");
            let mut rng = thread_rng();
            self.world_primitives.insert(wid, WorldPrimitive::new(rng.gen(), rng.gen()));
        }
        println!("Returning a world");
        *self.world_primitives.get(&wid)
        .expect("I trusted you, you let me down")
    }
}
