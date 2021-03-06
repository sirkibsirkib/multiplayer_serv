use super::SaverLoader;
use ::engine::game_state::locations::{Location,LocationPrimitive};
use ::identity::{LocationID};
use super::{WorldLoader,WorldPrimLoader};
use super::{Diff};
use super::super::super::game_state::worlds::zones::Zone;
use super::super::super::game_state::worlds::START_WORLD;
use ::utils::traits::*;
// use super::super::network::messaging::MsgToClient;



#[derive(Debug)]
pub struct LocationGuard {
    loc : Location,
    diffs : Vec<Diff>,
}


impl KnowsSavePrefix for Vec<Diff> {
    fn get_save_prefix() -> String {
        "loc_diffs".to_owned()
    }
}

impl LocationGuard {
    #[inline]
    pub fn get_location_primitive(&self) -> &LocationPrimitive {
        self.loc.get_location_primitive()
    }

    pub fn borrow_location(&self) -> &Location {
        &self.loc
    }

    pub fn apply_diff(&mut self, diff : Diff) -> Result<(),()> {
        if self.loc.apply_diff(diff).is_ok() {
            self.diffs.push(diff);
            Ok(())
        } else {
            Err(())
        }
    }


    pub fn save_to(&self, sl : &SaverLoader, lid : LocationID) {
        println!("saving loc lid:{:?} prim", lid);
        sl.save_with_key(self.loc.get_location_primitive(), lid).is_ok();
        println!("saving loc lid:{:?} diffs", lid);
        sl.save_with_key(
            & self.diffs,
            lid,
        ).is_ok();
    }

    pub fn load_from(sl : &SaverLoader, lid : LocationID, world_loader: &mut WorldLoader, wpl: &mut WorldPrimLoader) -> LocationGuard {
        match sl.load_with_key(lid) {
            Ok(prim) => { //found prim
                let diffs : Vec<Diff> = sl.load_with_key(lid)
                    .expect("prim ok but diffs not??");
                    //don't store diffs just yet. let loc_guard do that
                    //TODO move server_game_state into its own module
                let prim2 : LocationPrimitive = prim; //can't wait for type ascription
                let wid = prim2.wid;
                let w = world_loader.get_world(wid, wpl);
                let z : Zone = w.get_zone(prim2.zone_id).clone();
                let loc : Location = Location::generate_new(prim2, z);
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
                if lid == super::START_LOCATION_LID { //ok must be a new game
                    println!("Generating start location!");
                    LocationGuard {
                        // TODO check this doesnt make duplicates
                        loc : Location::generate_new(*super::START_LOC_PRIM, START_WORLD.get_zone(0).clone()),
                        diffs : vec![],
                    }
                } else { //nope! just missing savefile
                    panic!("MISSING SAVEFILE??");
                }
            },
        }
    }
}
