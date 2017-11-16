use super::{EntityID,WIDTH,HEIGHT,Location,Point,ScreenPoint,Dataset};
use super::piston_window::{PistonWindow,GenericEvent,clear,image,Transformed};
use std::time::{Instant,Duration};
use ::identity::{ObjectID};
use ::network::messaging::MsgToServer;

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

    pub fn zoom_out(&mut self) {
        self.vp.screen_meter_width /= 0.9;
    }

    pub fn zoom_in(&mut self) {
        self.vp.screen_meter_width *= 0.9;
    }

    // suggest vp == ViewPerspective::default_surface()
    pub fn new(eid : EntityID, location : Location, vp : ViewPerspective) -> View {
        View {
            eid : eid,
            location : location,
            vp : vp,
        }
    }

    fn translate_screenpt_relative_to(&self, screen_pt : ScreenPoint, center : Point) -> Point {
        let prim = self.location.get_location_primitive();
        let meter_to_pixels : f64 = WIDTH / self.vp.screen_meter_width;
        let cells_to_pixels : f64 = prim.cell_to_meters * meter_to_pixels;
        [
            center[0] + (-0.5 + (screen_pt[0] - (WIDTH / 2.0)) / cells_to_pixels) as i16,
            center[1] + (-0.5 + (screen_pt[1] - (HEIGHT / 2.0)) / cells_to_pixels) as i16,
        ]
    }

    pub fn translate_screenpt(&self, screen_pt : ScreenPoint) -> Option<Point> {
        self.location.point_of(self.eid)
        .map(|center| self.translate_screenpt_relative_to(screen_pt, center))
    }

    pub fn translate_pt_relative_to(&self, pt : Point, center : Point) -> ScreenPoint {
        let prim = self.location.get_location_primitive();
        let rel_pt = [pt[0] - center[0], pt[1] - center[1]];
        let meter_to_pixels : f64 = WIDTH / self.vp.screen_meter_width;
        let cells_to_pixels : f64 = prim.cell_to_meters * meter_to_pixels;
        [
            (WIDTH / 2.0) + (rel_pt[0] as f64 * cells_to_pixels),
            (HEIGHT / 2.0) + (rel_pt[1] as f64 * cells_to_pixels),
        ]
    }

    pub fn translate_pt(&self, pt : Point) -> Option<ScreenPoint> {
        self.location.point_of(self.eid)
        .map(|center| self.translate_pt_relative_to(pt, center))
    }

    #[inline]
    pub fn get_location(&self) -> &Location {
        &self.location
    }

    #[inline]
    pub fn get_location_mut(&mut self) -> &mut Location {
        &mut self.location
    }

    pub fn render_location<E>(
                       &self,
                       event : &E,
                       window : &mut PistonWindow,
                       dataset : &mut Dataset,
    ) where E : GenericEvent {
        window.draw_2d(event, |c, g| {
            clear([0.0, 0.0, 0.0, 1.0], g);
        });
        if let Some(center) = self.location.point_of(self.eid) {
            let mut missing_oid_assets : Vec<ObjectID> = vec![];
            window.draw_2d(event, |c, g| {
                for (oid, pt_set) in self.get_location().object_iterator() {
                    if missing_oid_assets.contains(&oid) {
                        continue;
                    }
                    if let Some(object_data) = dataset.object_dataset.get(*oid) {
                        let zoom = calc_zoom(
                            dataset.asset_manager.get_tex_width(object_data.aid),
                            self.vp.screen_meter_width,
                            object_data.width_meters,
                        );
                        let tex = dataset.asset_manager.get_texture_for(object_data.aid);
                        object_data.width_meters;
                        for screen_pt in pt_set.iter()
                        .map(|pt| self.translate_pt_relative_to(*pt, center))
                        .filter(is_on_screen) {
                            image(tex, c.transform.trans(screen_pt[0], screen_pt[1]).zoom(zoom), g);
                        }
                    } else {
                        missing_oid_assets.push(*oid);
                    }
                    if ! missing_oid_assets.is_empty() {
                        let now = Instant::now();
                        if dataset.data_requests_supressed_until.ins < now {
                            for oid in missing_oid_assets.iter() {
                                println!("Requesting OID {:?}'s data", &oid);
                                dataset.outgoing_request_cache.push(
                                    MsgToServer::RequestObjectData(*oid)
                                ); // request to populate asset manager
                            }
                            dataset.data_requests_supressed_until.ins = now + dataset.data_requests_supressed_until.setdur;
                        }
                    }
                }
            });



            let mut missing_eid_assets : Vec<ObjectID> = vec![];
            window.draw_2d(event, |c, g| {
                for (eid, pt) in self.get_location().entity_iterator() {
                    if missing_eid_assets.contains(&eid) {
                        continue;
                    }
                    if let Some(entity_data) = dataset.entity_dataset.get(*eid) {
                        let zoom = calc_zoom(
                            dataset.asset_manager.get_tex_width(entity_data.aid),
                            self.vp.screen_meter_width,
                            entity_data.width_meters,
                        );
                        let tex = dataset.asset_manager.get_texture_for(entity_data.aid);
                        let screen_pt = self.translate_pt_relative_to(*pt, center);
                        if is_on_screen(&screen_pt) {
                            image(tex, c.transform
                                .trans(screen_pt[0], screen_pt[1]).zoom(zoom), g);
                        }
                    } else {
                        missing_eid_assets.push(*eid);
                    }
                    if ! missing_eid_assets.is_empty() {
                        let now = Instant::now();
                        if dataset.data_requests_supressed_until.ins < now {
                            for eid in missing_eid_assets.iter() {
                                println!("Requesting EID {:?}'s data", &eid);
                                dataset.outgoing_request_cache.push(
                                    MsgToServer::RequestEntityData(*eid)
                                ); // request to populate asset manager
                            }
                            dataset.data_requests_supressed_until.ins = now + dataset.data_requests_supressed_until.setdur;
                        }
                    }
                }
            });
        }
    }
}

#[inline]
fn calc_zoom(sprite_pixels : u32, view_width_meters : f64, rendered_width_meters : f64) -> f64 {
    WIDTH
    / sprite_pixels as f64
    / view_width_meters
    * rendered_width_meters
}

fn is_on_screen(sp : &ScreenPoint) -> bool {
    sp[0] >= 0.0 && sp[0] < WIDTH
    && sp[1] >= 0.0 && sp[1] < HEIGHT
}
