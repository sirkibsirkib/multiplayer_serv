use super::{EntityID,WIDTH,HEIGHT,Location,Point};

pub struct View {
    h_rad_units : f64,
    v_rad_units : f64,
    eid : EntityID,
    location : Location,
    zoom : f64,
}

impl View {
    pub fn translate_screenpt(&self, screen_pt : [f64;2]) -> Point {
        let prim = self.location.get_location_primitive();
        [
            (screen_pt[0]/WIDTH * prim.cells_wide as f64) as i16,
            (screen_pt[1]/HEIGHT * prim.cells_high as f64) as i16,
        ]
        //[0,0] is topleft  [WIDTH,HEIGHT] is top right
        //TODO complex shit
        // let relative = [screen_pt[0]-0.5, screen_pt[1]-0.5];
        // let e_at : Point = self.location.point_of(self.eid).expect("VIEW CANT FIND");
        // let cell_width = self.location.get_cell_width();
        // [
        //     (relative[0] * self.h_rad_units/cell_width as f64) as i16 + e_at[0],
        //     (relative[1] * self.v_rad_units/cell_width as f64) as i16 + e_at[1],
        // ]
    }

    //TODO what happens when outside screen?
    pub fn translate_pt(&self, pt : Point) -> [f64;2] {
        //TODO make not stupid
        let prim = self.location.get_location_primitive();
        [
            pt[0] as f64 / prim.cells_wide as f64 * WIDTH,
            pt[1] as f64 / prim.cells_high as f64 * HEIGHT,
        ]
    }

    #[inline]
    pub fn get_location(&self) -> &Location {
        &mut self.location
    }

    #[inline]
    pub fn get_location_mut(&mut self) -> &mut Location {
        &mut self.location
    }
}