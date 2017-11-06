use std::time::{Instant,Duration};
use std::fs::File;
use std::io::{Read,Write};
use std::collections::HashMap;
use super::SaverLoader;


pub struct UniversalCoord {
    lid : LocationID,
    x : u32,
    y : u32,
}

#[derive(Debug)]
pub struct Location {

}

impl Location {
    fn load_from_file(path : &str) -> Location {
        let mut data = String::new();
        let mut f = File::open(path).expect("Unable to read Location");
        f.read_to_string(&mut data).expect("Unable to read string");
        Location{

        }
    }

    fn write_to_file(&self, path : &str) {
        let data = "Some data!";
        let mut f = File::create(path).expect("Unable to create file");
        f.write_all(data.as_bytes()).expect("Unable to write data");
    }
}

type LocationID = u32;

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

    pub fn load(&mut self, lid : LocationID) {
        if ! self.foreground.contains_key(& lid) {
            if let Some(timestamped_loc) = self.background.remove(& lid) {
                // upgrade background --> foreground
                println!("Promoting background LID {:?}", &lid);
                self.foreground.insert(lid, timestamped_loc.location);
            } else {
                //fresh load from file
                println!("Loading background LID {:?}", &lid);
                let l = Location::load_from_file(& format!("{}", &lid));
                self.foreground.insert(lid, l);
            }
        }
    }

    pub fn unload_overdue_backgrounds(&mut self) {
        let mut remove_lids = vec![];
        for (k, v) in self.background.iter_mut() {
            if v.loaded_at.elapsed() > self.background_retention {
                //unload
                v.location.write_to_file(&format!("{}", &k));
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
