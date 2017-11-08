use std::time::{Instant,Duration};
use std::io::Write;
use std::collections::HashMap;
use super::SaverLoader;
use super::game_state::{Entity,Point,Location};
use super::super::identity::{EntityID,LocationID};
use std::collections::HashSet;
use super::ClientID;
use super::super::network::messaging::MsgToClient;

pub const START_LOCATION : LocationID = 0;


#[derive(Debug)]
struct TimestampedLocation {
    location : Location,
    loaded_at : Instant,
}

struct LocationGuard {
    loc : Location,
    diffs : Vec<MsgToClient>,
}

pub struct LocationLoader {
    sl : SaverLoader,
    background_retention : Duration,

    subscriptions : HashMap<LocationID,HashSet<ClientID>>,
    background : HashMap<LocationID, TimestampedLocation>,
    foreground : HashMap<LocationID, Location>,

    last_simulated : HashMap<LocationID,Instant>,
    last_backgrounded : HashMap<LocationID,Instant>,
}

impl LocationLoader {
    pub fn save_all_locations(&self) {
        for (lid, loc) in self.foreground_background_iter() {
            println!("saving loc {:?}", lid);
            self.sl.save_me(loc, & Location::filepath(*lid));
        }
    }

    pub fn new(background_retention : Duration, sl : SaverLoader) -> LocationLoader {
        LocationLoader {
            sl : sl,
            subscriptions :  HashMap::new(),
            background : HashMap::new(),
            foreground : HashMap::new(),

            // when it unloads a file, it logs a time. it will return the duration since then until you consume() it
            last_simulated : HashMap::new(),
            last_backgrounded : HashMap::new(),
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

    pub fn load_foreground(&mut self, lid : LocationID) -> &Location {
        self.load_foreground_mut(lid)
    }

    pub fn load_foreground_mut(&mut self, lid : LocationID) -> &mut Location {
        if ! self.foreground.contains_key(& lid) {
            if let Some(timestamped_loc) = self.background.remove(& lid) {
                // upgrade background --> foreground
                println!("Promoting background LID {:?}", &lid);
                self.foreground.insert(lid, timestamped_loc.location);
            } else {
                //fresh load from file
                println!("Loading background LID {:?}", &lid);
                match self.sl.load_me(& Location::filepath(lid)){
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
                self.sl.save_me(k, & Location::filepath(*k));
                remove_lids.push(*k);
            }
        }
        let nowish = Instant::now();
        for lid in remove_lids {
            self.background.remove(&lid);
            println!("Unloading background LID{:?}", lid);
            self.last_simulated.insert(lid, nowish);
        }
    }

    pub fn loaded(&self, lid : LocationID) -> bool {
        self.foreground.contains_key(& lid)
        || self.background.contains_key(& lid)
    }

    pub fn foreground_iter<'a>(&'a self) -> Box<Iterator<Item=(&LocationID, &Location)> + 'a> {
        Box::new(
            self.foreground.iter()
        )
    }

    pub fn foreground_background_iter<'a>(&'a self) -> Box<Iterator<Item=(&LocationID, &Location)> + 'a> {
        Box::new(
            self.foreground_iter().chain(self.background_iter())
        )
    }

    pub fn foreground_iter_mut<'a>(&'a mut self) -> Box<Iterator<Item=(&LocationID, &mut Location)> + 'a> {
        Box::new(
            self.foreground.iter_mut()
        )
    }

    pub fn background_iter<'a>(&'a self) -> Box<Iterator<Item=(&LocationID, &Location)> + 'a> {
        Box::new(
            self.background.iter()
            .map(|x| (x.0, &x.1.location))
        )
    }
}
