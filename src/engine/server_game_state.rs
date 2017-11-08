use std::time::{Instant,Duration};
use std::io::Write;
use std::collections::HashMap;
use super::SaverLoader;
use super::game_state::{EntityID,Entity,Point,LocationID,Location};
use std::collections::HashSet;
use super::super::network::ClientID;
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
            self.last_simulated.insert(lid, nowish);
        }
    }

    pub fn loaded(&self, lid : LocationID) -> bool {
        self.foreground.contains_key(& lid)
        || self.background.contains_key(& lid)
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
