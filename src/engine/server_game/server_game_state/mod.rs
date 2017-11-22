use std::time::{Instant,Duration};
use std::collections::HashMap;
use super::SaverLoader;
use ::engine::game_state::locations::{Location,LocationPrimitive};
use ::identity::{LocationID,SuperSeed};
use super::{ClientID,Diff};
use ::identity::ClientIDSet;


pub const START_LOCATION_LID : LocationID = 0;

mod loc_guard;
use self::loc_guard::LocationGuard;

pub struct LocationLoader {
    sl : SaverLoader,
    background_retention : Duration,

    subscriptions : HashMap<LocationID,ClientIDSet>,
    background : HashMap<LocationID, LocationGuard>,
    foreground : HashMap<LocationID, LocationGuard>,

    last_simulated : HashMap<LocationID,Instant>,
    last_backgrounded : HashMap<LocationID,Instant>,
}



pub fn start_location() -> Location {
    let start_loc_prim = LocationPrimitive {
        cells_wide : 50,
        cells_high : 50,
        cell_to_meters : 1.0, //in meters
        super_seed : START_LOCATION_LID as SuperSeed,
    };
    Location::new(start_loc_prim)
}


impl LocationLoader {

    pub fn borrow_location(&mut self, lid : LocationID) -> &Location {
        let loc_guard = if self.load_at_least_background(lid) {
            self.background.get_mut(&lid).expect("borrow be in BG")
        } else {
            self.foreground.get_mut(&lid).expect("borrow be in FG, ye")
        };
        loc_guard.borrow_location()
    }

    pub fn apply_diff_to(&mut self, lid : LocationID, diff : Diff, must_be_foreground : bool) -> Result<(),()> {
        let loc_guard = if must_be_foreground {
            self.load_foreground(lid);
            self.foreground.get_mut(&lid).expect("must be in FG")
        } else {
            if self.load_at_least_background(lid) {
                self.background.get_mut(&lid).expect("must be in BG")
            } else {
                self.foreground.get_mut(&lid).expect("must be in FG, ye")
            }
        };
        loc_guard.apply_diff(diff)
    }

    pub fn save_all_locations(&self) {
        for lid in self.foreground_iter() {
            let loc_guard = self.foreground.get(&lid).expect("wtf you said its in foreground");
            loc_guard.save_to(&self.sl, *lid);
        }
        for lid in self.background_iter() {
            let loc_guard = self.background.get(&lid).expect("wtf you said its in background");
            loc_guard.save_to(&self.sl, *lid);
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

    pub fn client_subscribe(&mut self, cid : ClientID, lid : LocationID) {
        if let Some(set) = self.subscriptions.get_mut(&lid) {
            set.set(cid,true);
            return;
        }
        let mut x = ClientIDSet::new();
        x.set(cid,true);
        self.subscriptions.insert(lid, x);
        // 0->1 subs. gotta foreground!
        self.load_foreground(lid);
    }

    pub fn client_unsubscribe(&mut self, cid : ClientID, lid : LocationID) {
        let mut flag = false;
        if let Some(ref mut set) = self.subscriptions.get_mut(&lid) {
            set.set(cid,false);
            if set.is_empty() {
                flag = true;
            }
        } // else subs 0->0 who cares
        if flag {
            // 1->0 subs. gotta foreground!
            self.unforeground(lid);
        }
    }

    pub fn subscribers_exist_for(&self, lid : LocationID) -> bool {
        self.subscriptions.contains_key(&lid)
    }

    pub fn get_subscriptions_for(&self, lid : LocationID) -> ClientIDSet {
        if let Some(x) = self.subscriptions.get(&lid) {
            x.clone()
        } else {
            ClientIDSet::new()
        }
    }

    pub fn is_subscribed(&self, cid : ClientID, lid : LocationID) -> bool {
        if let Some(set) = self.subscriptions.get(&lid) {
            set.get(cid)
        } else {
            false
        }
    }

    fn unforeground(&mut self, lid : LocationID) {
        if let Some(x) = self.foreground.remove(&lid) {
            println!("Demoting LID {:?} to background", &lid);
            self.background.insert(lid, x);
            self.last_backgrounded.insert(lid, Instant::now());
        }
    }

    pub fn consume_time_since_last_sim(&mut self, lid : LocationID) -> Option<Duration> {
        self.last_simulated.remove(&lid)
        .map(|t| t.elapsed())
    }

    /*
    if unloaded, loads to background.
    returns TRUE if its in background, false if its in FOREGROUND
    */
    fn load_at_least_background(&mut self, lid : LocationID) -> bool {
        if self.foreground.contains_key(& lid) {
            false
        } else {
            if ! self.foreground.contains_key(& lid) {
                println!("fresh file load for loc with LID {:?}", &lid);
                let loc_guard = LocationGuard::load_from(&self.sl, lid);
                if let Some(dur) = self.consume_time_since_last_sim(lid) {
                    //TODO alter loc_guard to represent `dur` time passing
                }
                self.background.insert(
                    lid,
                    loc_guard,
                );
                self.last_backgrounded.insert(lid, Instant::now());
            }
            true
        }
    }

    //if not in foreground, loads to foreground
    fn load_foreground(&mut self, lid : LocationID) {
        if ! self.foreground.contains_key(& lid) {
            //it's not already loaded in foreground
            self.load_at_least_background(lid);
            let loc = self.background.remove(& lid).expect("IT should be in background!");
            // upgrade background --> foreground
            println!("Promoting background LID {:?}", &lid);
            self.foreground.insert(lid, loc);
            self.last_backgrounded.remove(&lid); //no longer backgrounded
        }
    }

    pub fn print_status(&self) {
        println!("LocLoader status: {{", );
        for lid in self.foreground_iter() {
            println!("\tFG {:?} cid subs: {:?}", lid, &self.subscriptions.get(lid));
        }
        println!("\t---");
        for lid in self.background_iter() {
            println!("\tBG {:?} time bg'd: {:?}", lid, &self.last_backgrounded.get(lid).unwrap().elapsed());
        }
        println!("}}");
    }

    pub fn unload_overdue_backgrounds(&mut self) {
        let mut remove_lids = vec![];
        for (lid, v) in self.background.iter_mut() {
            println!("considering unloading {:?}", &lid);
            if self.last_backgrounded.get(lid).expect("no last backgrounded??").elapsed() > self.background_retention {
                //save to file
                v.save_to(&self.sl, *lid);
                remove_lids.push(*lid);
            }
        }
        let nowish = Instant::now();
        for lid in remove_lids {
            //unload from background map
            self.background.remove(&lid);
            println!("Unloading background LID {:?}", lid);
            //marking as "last simulated" around this time
            self.last_simulated.insert(lid, nowish);
        }
    }

    pub fn foreground_iter<'a>(&'a self) -> Box<Iterator<Item=&LocationID> + 'a> {
        Box::new(
            self.foreground.keys()
        )
    }

    pub fn background_iter<'a>(&'a self) -> Box<Iterator<Item=&LocationID> + 'a> {
        Box::new(
            self.background.keys()
        )
    }
}
