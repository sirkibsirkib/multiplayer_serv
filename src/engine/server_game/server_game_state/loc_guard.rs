use super::SaverLoader;
use ::engine::game_state::locations::{Location,LocationPrimitive};
use ::identity::{LocationID};
use super::{Diff};
use ::engine::primitives::*;
// use super::super::network::messaging::MsgToClient;



#[derive(Debug)]
pub struct LocationGuard {
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
        sl.save_me(
            & self.loc.get_location_primitive(),
            & location_primitive_save_path(lid),
        ).is_ok();

        println!("saving loc lid:{:?} diffs", lid);
        sl.save_me(
            & self.diffs,
            & location_diffs_save_path(lid),
        ).is_ok();
    }

    pub fn load_from(sl : &SaverLoader, lid : LocationID) -> LocationGuard {
        match sl.load_me(& location_primitive_save_path(lid)) {
            Ok(prim) => { //found prim
                let diffs : Vec<Diff> = sl.load_me(& location_diffs_save_path(lid))
                    .expect("prim ok but diffs not??");
                    //don't store diffs just yet. let loc_guard do that
                    //TODO move server_game_state into its own module
                let prim2 : LocationPrimitive = prim; //can't wait for type ascription
                let loc : Location = prim2.generate_new();
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
                        loc : super::start_location(),
                        diffs : vec![],
                    }
                } else { //nope! just missing savefile
                    panic!("MISSING SAVEFILE??");
                }
            },
        }
    }
}
