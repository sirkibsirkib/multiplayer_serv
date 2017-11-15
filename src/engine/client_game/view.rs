use super::{EntityID,WIDTH,HEIGHT,Location,Point,ScreenPoint};

pub struct View {
    eid : EntityID,
    location : Location,
    vp : ViewPerspective,
}

pub struct ViewPerspective {
    screen_meter_width : f64,
}

impl ViewPerspective {
    pub const DEFAULT_SURFACE : ViewPerspective = ViewPerspective {
        screen_meter_width : 90.0,
    };
}

impl View {

    // suggest vp == ViewPerspective::default_surface()
    pub fn new(eid : EntityID, location : Location, vp : ViewPerspective) -> View {
        View {
            eid : eid,
            location : location,
            vp : vp,
        }
    }

    pub fn translate_screenpt(&self, screen_pt : ScreenPoint) -> Point {


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
    pub fn translate_pt(&self, pt : Point) -> Option<ScreenPoint> {
        let prim = self.location.get_location_primitive();
        if let Some(center) = self.location.point_of(self.eid) {
            let rel_pt = [pt[0] - center[0], pt[1] - center[1]];
            let meter_to_pixels : f64 = WIDTH / self.vp.screen_meter_width;
            let cells_to_pixels : f64 = prim.cell_to_meters * meter_to_pixels;
            let q = [
                (WIDTH / 2.0) + (rel_pt[0] as f64 * cells_to_pixels),
                (HEIGHT / 2.0) + (rel_pt[1] as f64 * cells_to_pixels),
            ];
            println!("{:?} => {:?}", rel_pt, q);
            Some(q)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_location(&self) -> &Location {
        &self.location
    }

    #[inline]
    pub fn get_location_mut(&mut self) -> &mut Location {
        &mut self.location
    }
}
