use std::time::{Instant,Duration};
use std::io::Write;
use std::collections::HashMap;
use super::SaverLoader;
use super::game_state::{Entity,Point,Location,LocationPrimitive};
use super::super::identity::{EntityID,LocationID};
use std::collections::HashSet;
use super::{ClientID,Diff};
use super::super::network::messaging::MsgToClient;

pub const START_LOCATION : LocationID = 0;



struct LocationGuard {
    loc : Location,
    diffs : Vec<Diff>,
}


fn location_primitive_save_path(lid : LocationID) -> String {
    format!("locations/loc_{}_prim.lel", lid)
}

fn location_diffs_save_path(lid : LocationID) -> String {
    format!("locations/loc_{}_diffs.lel", lid)
}

impl LocationGuard {
    fn entity_iterator<'a>(&'a self) -> Box<Iterator<Item=(&EntityID,&Point)> + 'a> {
        self.loc.entity_iterator()
    }

    fn apply_diff(&mut self, diff : Diff) {
        //APPLY the diff
        self.loc.apply_diff(diff);
        //STORE the diff
        self.diffs.push(diff);
    }

    fn save_to(&self, sl : &SaverLoader, lid : LocationID) {
        println!("saving loc lid:{:?} prim", lid);
        sl.save_me(
            & self.loc.get_location_primitive(),
            & location_primitive_save_path(lid),
        );

        println!("saving loc lid:{:?} diffs", lid);
        sl.save_me(
            & self.diffs,
            & location_diffs_save_path(lid),
        );
    }

    fn load_from(sl : &SaverLoader, lid : LocationID) -> LocationGuard {
        match sl.load_me(& location_primitive_save_path(lid)) {
            Ok(prim) => { //found prim
                let diffs : Vec<Diff> = sl.load_me(& location_diffs_save_path(lid))
                    .expect("prim ok but diffs not??");
                    //don't store diffs just yet. let loc_guard do that
                    //TODO move server_game_state into its own module
                let mut loc = Location::new(prim);
                let mut loc_guard = LocationGuard {
                    loc : loc,
                    diffs : vec![],
                };
                //apply all diffs in trn
                for diff in diffs {
                    loc_guard.apply_diff(diff);
                }
                loc_guard
            },
            Err(_) => { // couldn't find savefile!
                if lid == START_LOCATION { //ok must be a new game
                    println!("Generating start location!");
                    LocationGuard {
                        loc : Location::start_location(),
                        diffs : vec![],
                    }
                } else { //nope! just missing savefile
                    panic!("MISSING SAVEFILE??");
                }
            },
        }
    }
}

pub struct LocationLoader {
    sl : SaverLoader,
    background_retention : Duration,

    subscriptions : HashMap<LocationID,HashSet<ClientID>>,
    background : HashMap<LocationID, LocationGuard>,
    foreground : HashMap<LocationID, LocationGuard>,

    last_simulated : HashMap<LocationID,Instant>,
    last_backgrounded : HashMap<LocationID,Instant>,
}


impl LocationLoader {

    pub fn apply_diff_to(&mut self, lid : LocationID, diff : Diff, must_be_foreground : bool) {
        let mut loc_guard = if must_be_foreground {
            self.load_foreground(lid);
            self.foreground.get_mut(&lid).expect("must be in FG")
        } else {
            if self.load_at_least_background(lid) {
                self.background.get_mut(&lid).expect("must be in BG")
            } else {
                self.foreground.get_mut(&lid).expect("must be in FG, ye")
            }
        };
        loc_guard.apply_diff(diff);
    }

    // fn loader_find_locguard(lid : LocationID) -> &mut LocationGuard {
    //
    // }

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

    pub fn unforeground(&mut self, lid : LocationID) {
        if let Some(x) = self.foreground.remove(&lid) {
            println!("Demoting LID {:?} to background", &lid);
            self.background.insert(lid, x);
        }
    }

    pub fn consume_time_since_last_sim(&mut self, lid : LocationID) -> Option<Duration> {
        if let Some(time) = self.last_simulated.remove(&lid) {
            Some(time.elapsed())
        } else {
            None
        }
    }

    /*
    if unloaded, loads to background.
    returns TRUE if its in background, false if its in FOREGROUND
    */
    pub fn load_at_least_background(&mut self, lid : LocationID) -> bool {
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
            }
            true
        }
    }

    //if not in foreground, loads to foreground
    pub fn load_foreground(&mut self, lid : LocationID) {
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

    pub fn unload_overdue_backgrounds(&mut self) {
        let mut remove_lids = vec![];
        for (lid, v) in self.background.iter_mut() {
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

    // pub fn loaded(&self, lid : LocationID) -> bool {
    //     self.foreground.contains_key(& lid)
    //     || self.background.contains_key(& lid)
    // }

    pub fn entity_iterator<'a>(&'a mut self, lid : LocationID) -> Box<Iterator<Item=(&EntityID,&Point)> + 'a> {
        let loc_guard = if self.load_at_least_background(lid) {
            self.background.get(&lid).expect("must be in BG")
        } else {
            self.foreground.get(&lid).expect("must be in FG, ye")
        };
        Box::new(
            loc_guard.entity_iterator()
        )
    }

    pub fn foreground_iter<'a>(&'a self) -> Box<Iterator<Item=&LocationID> + 'a> {
        Box::new(
            self.foreground.keys()
        )
    }



    pub fn foreground_background_iter<'a>(&'a self) -> Box<Iterator<Item=&LocationID> + 'a> {
        Box::new(
            self.foreground_iter().chain(self.background_iter())
        )
    }

    pub fn background_iter<'a>(&'a self) -> Box<Iterator<Item=&LocationID> + 'a> {
        Box::new(
            self.background.keys()
        )
    }
}
