use ::std::time::{Duration};
use ::engine::entities::{EntityDataSet};
use ::engine::objects::{ObjectDataSet};
use super::server_game_state::{LocationLoader};
use std::collections::HashMap;
use super::SaverLoader;
use ::identity::{EntityID,ClientID,LocationID};

// subset of data that needs to be persistent between games. ie loaded and saved
#[derive(Serialize,Deserialize,Debug)]
struct Persistent {
    next_eid : EntityID,
    cid_to_controlling : HashMap<ClientID, (EntityID,LocationID)>,
    entity_data_set : EntityDataSet,
    object_data_set : ObjectDataSet,
}

impl Persistent {
    const SAVE_PATH : &'static str =  "./persistent_server_data.lel";

    fn start_and_load(sl : &SaverLoader) -> Persistent {
        match sl.load_me(Self::SAVE_PATH) {
            Ok(x) => {
                println!("Successfully loaded persistent server data");
                x
            },
            Err(_) => {
                println!("Failed to load persistent server data");
                Persistent {
                    next_eid : 0,
                    cid_to_controlling : HashMap::new(),
                    entity_data_set : EntityDataSet::new(),
                    object_data_set : ObjectDataSet::new(),
                }
            }
        }
    }
}

//struct for keeping track of RESOURCES. AIDs entity data, etc.
pub struct ServerResources {
    persistent : Persistent,
    location_loader : LocationLoader,
    sl : SaverLoader,
}

impl ServerResources {
    pub fn start_and_load(sl : SaverLoader) -> ServerResources {
        ServerResources {
            persistent : Persistent::start_and_load(&sl),
            location_loader : LocationLoader::new(Duration::new(10,0), sl.clone()),
            sl : sl,
        }
    }


    pub fn save_all(&mut self) {
        self.sl.save_me(&self.persistent, Persistent::SAVE_PATH)
        .expect("couldn't save persistent server data!");
        self.location_loader.unload_overdue_backgrounds();
        self.location_loader.save_all_locations();
        self.location_loader.print_status();
    }

    /////////////////////////////////////////////////// BORROWS

    #[inline]
    pub fn borrow_location_loader(&self) -> &LocationLoader {
        &self.location_loader
    }

    #[inline]
    pub fn borrow_object_data_set(&self) -> &ObjectDataSet {
        &self.persistent.object_data_set
    }

    #[inline]
    pub fn borrow_entity_data_set(&self) -> &EntityDataSet {
        &self.persistent.entity_data_set
    }

    ///////////////////////////////////////////////////// MUT BORROWS

    #[inline]
    pub fn borrow_mut_location_loader(&mut self) -> &mut LocationLoader {
        &mut self.location_loader
    }

    #[inline]
    pub fn borrow_mut_object_data_set(&mut self) -> &mut ObjectDataSet {
        &mut self.persistent.object_data_set
    }

    #[inline]
    pub fn borrow_mut_entity_data_set(&mut self) -> &mut EntityDataSet {
        &mut self.persistent.entity_data_set
    }
}
