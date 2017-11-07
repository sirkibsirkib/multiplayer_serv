use std::time::{Instant,Duration};
use std::fs::File;
use std::io::{Read,Write};
use std::collections::HashMap;
use super::SaverLoader;
use super::game_state::{EntityID,Entity,Point};

pub type LocationID = u32;
pub const START_LOCATION : LocationID = 0;

pub struct UniversalCoord {
    lid : LocationID,
    x : u32,
    y : u32,
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


#[derive(Debug)]
struct TimestampedLocation {
    location : Location,
    loaded_at : Instant,
}

pub struct LocationLoader {
    sl : SaverLoader,
    background : HashMap<LocationID, TimestampedLocation>,
    foreground : HashMap<LocationID, Location>,
    unloaded_since : HashMap<LocationID, Instant>,
    background_retention : Duration,
}

impl LocationLoader {
    pub fn new(background_retention : Duration, sl : SaverLoader) -> LocationLoader {
        LocationLoader {
            sl : sl,
            background : HashMap::new(),
            foreground : HashMap::new(),

            // when it unloads a file, it logs a time. it will return the duration since then until you consume() it
            unloaded_since : HashMap::new(),
            background_retention : background_retention,
        }
    }

    pub fn unforeground(&mut self, lid : LocationID) {
        if let Some(x) = self.foreground.remove(&lid) {
            println!("Demoting LID {:?} to background", &lid);
            self.background.insert(
                lid,
                TimestampedLocation{
                    location : x,
                    loaded_at : Instant::now(),
                }
            );
        }
    }

    pub fn get_mut_foreground(&mut self, lid : LocationID) -> &mut Location {
        self.foreground.get_mut(&lid).unwrap()
    }

    pub fn get_foreground(&self, lid : LocationID) -> &Location {
        self.foreground.get(&lid).unwrap()
    }

    pub fn load(&mut self, lid : LocationID) -> &mut Location {
        if ! self.foreground.contains_key(& lid) {
            if let Some(timestamped_loc) = self.background.remove(& lid) {
                // upgrade background --> foreground
                println!("Promoting background LID {:?}", &lid);
                self.foreground.insert(lid, timestamped_loc.location);
            } else {
                //fresh load from file
                println!("Loading background LID {:?}", &lid);
                match self.sl.load_me(& Location::filename(lid)){
                    Ok(l) => {
                        self.foreground.insert(lid, l);
                    },
                    Err(_) => {
                        if lid == START_LOCATION {
                            println!("Generating start location!");
                            self.foreground.insert(lid, Location::start_location());
                        }
                    }
                }
            }
        }
        self.foreground.get_mut(&lid).unwrap()
    }

    pub fn unload_overdue_backgrounds(&mut self) {
        let mut remove_lids = vec![];
        for (k, v) in self.background.iter_mut() {
            if v.loaded_at.elapsed() > self.background_retention {
                //unload
                self.sl.save_me(k, & Location::filename(*k));
                remove_lids.push(*k);
            }
        }
        let nowish = Instant::now();
        for lid in remove_lids {
            self.background.remove(&lid);
            println!("Unloading background LID{:?}", lid);
            self.unloaded_since.insert(lid, nowish);
        }
    }

    pub fn loaded(&self, lid : LocationID) -> bool {
        self.foreground.contains_key(& lid)
        || self.background.contains_key(& lid)
    }

    pub fn consume_unloaded_duration(&mut self, lid : LocationID) -> Option<Duration> {
        if let Some(dur) = self.unloaded_since.remove(&lid) {
            Some(dur.elapsed())
        } else {
            return None;
        }
    }


    pub fn foreground_iter_mut<'a>(&'a mut self) -> Box<Iterator<Item=(&mut Location)> + 'a> {
        Box::new(
            self.foreground.values_mut().map(|x| &mut(*x))
        )
    }

    pub fn background_iter_mut<'a>(&'a mut self) -> Box<Iterator<Item=(&mut Location)> + 'a> {
        Box::new(
            self.background.values_mut().map(|x| &mut x.location)
        )
    }
}
