use super::{EntityID,WIDTH,HEIGHT,Location,MyData};
use super::ClientResources;
use super::AssetManager;
use ::points::*;
use super::piston_window::{PistonWindow,GenericEvent,clear,image,Transformed};
use std::time::{Instant,Duration};
use ::identity::*;
use ::network::messaging::MsgToServer;
use super::asset_manager::HardcodedAssets;
use super::piston_window::{G2dTexture,Texture,TextureSettings,Flip};
use super::piston_window::ImageSize;

pub struct View {
    pub eid : EntityID,
    pub lid: LocationID,
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

lazy_static! {
    static ref SCREEN_MIDDLE : CPoint2 = CPoint2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
}

impl View {

    pub fn zoom_out(&mut self) {
        self.vp.screen_meter_width /= 0.9;
    }

    pub fn zoom_in(&mut self) {
        self.vp.screen_meter_width *= 0.9;
    }

    // suggest vp == ViewPerspective::default_surface()
    pub fn new(eid : EntityID, lid: LocationID, vp : ViewPerspective) -> View {
        View {
            eid : eid,
            lid: lid,
            vp : vp,
        }
    }

    fn translate_screenpt_relative_to(&self, screen_pt : CPoint2, center : DPoint2, loc: &Location) -> DPoint2 {
        let prim = loc.get_location_primitive();
        let meter_to_pixels : f64 = WIDTH / self.vp.screen_meter_width;
        let cells_to_pixels : f64 = prim.cell_to_meters * meter_to_pixels;
        center + (screen_pt - *SCREEN_MIDDLE).div_scale(cells_to_pixels as f32).discrete()
        // [
        //     center[0] + (-0.5 + (screen_pt[0] - (WIDTH / 2.0)) / cells_to_pixels) as i16,
        //     center[1] + (-0.5 + (screen_pt[1] - (HEIGHT / 2.0)) / cells_to_pixels) as i16,
        // ]
    }

    pub fn translate_screenpt(&self, screen_pt : CPoint2, loc: &Location) -> Option<DPoint2> {
        loc.point_of(self.eid)
        .map(|center| self.translate_screenpt_relative_to(screen_pt, center, loc))
    }

    pub fn translate_pt_relative_to(&self, pt : DPoint2, center : DPoint2, loc: &Location) -> CPoint2 {
        let prim = loc.get_location_primitive();
        // let rel_pt = [pt[0] - center[0], pt[1] - center[1]];
        let rel_pt = pt - center;
        let meter_to_pixels : f64 = WIDTH / self.vp.screen_meter_width;
        let cells_to_pixels : f64 = prim.cell_to_meters * meter_to_pixels;
        *SCREEN_MIDDLE + (rel_pt.continuous().scale(cells_to_pixels as f32))
        // [
        //     (WIDTH / 2.0) + (rel_pt[0] as f64 * cells_to_pixels),
        //     (HEIGHT / 2.0) + (rel_pt[1] as f64 * cells_to_pixels),
        // ]
    }

    pub fn translate_pt(&self, pt : DPoint2, loc: &Location) -> Option<CPoint2> {
        loc.point_of(self.eid)
        .map(|center| self.translate_pt_relative_to(pt, center, loc))
    }

    pub fn render_location_objects<E>(
                       &self,
                       event : &E,
                       window : &mut PistonWindow,
                       client_resources: &mut ClientResources,
                       asset_manager: &mut AssetManager,
                       loc: &Location,
    ) where E : GenericEvent {
        if let Some(center) = loc.point_of(self.eid) {
            let mut missing_oid_assets : Vec<ObjectID> = vec![];
            window.draw_2d(event, |c, g| {
                for (oid, pt_set) in loc.object_iterator() {
                    if missing_oid_assets.contains(&oid) {continue}
                    if let Ok(object_data) = client_resources.get_object(*oid) {
                        let zoom = calc_zoom(
                            asset_manager.get_tex_width(object_data.aid),
                            self.vp.screen_meter_width,
                            object_data.width_meters,
                        );
                        let tex = asset_manager.get_texture_for(object_data.aid);
                        object_data.width_meters;
                        for screen_pt in pt_set.iter()
                        .map(|pt| self.translate_pt_relative_to(*pt, center, loc))
                        .filter(is_on_screen)
                        {
                            image(tex, c.transform.trans(screen_pt.x as f64, screen_pt.y as f64).zoom(zoom), g);
                        }
                    } else {
                        missing_oid_assets.push(*oid);
                    }
                }
            });
        }
    }

    pub fn render_location_entities<E>(
                       &self,
                       event : &E,
                       window : &mut PistonWindow,
                       client_resources: &mut ClientResources,
                       asset_manager: &mut AssetManager,
                       loc: &Location,
    ) where E : GenericEvent {
        if let Some(center) = loc.point_of(self.eid) {
            let mut missing_eid_assets : Vec<ObjectID> = vec![];
            window.draw_2d(event, |c, g| {
                for (eid, pt) in loc.entity_iterator() {
                    if missing_eid_assets.contains(&eid) {
                        continue;
                    }
                    if let Ok(entity_data) = client_resources.get_entity(*eid) {
                        let zoom = calc_zoom(
                            asset_manager.get_tex_width(entity_data.aid),
                            self.vp.screen_meter_width,
                            entity_data.width_meters,
                        );
                        let tex = asset_manager.get_texture_for(entity_data.aid);
                        let screen_pt = self.translate_pt_relative_to(*pt, center, loc);
                        if is_on_screen(&screen_pt) {
                            image(tex, c.transform
                                .trans(screen_pt.x as f64, screen_pt.y as f64).zoom(zoom), g);
                        }
                    } else {
                        missing_eid_assets.push(*eid);
                    }
                }
            });
        }
    }

    pub fn clear_window<E>(event : &E, window : &mut PistonWindow) where E : GenericEvent {
        window.draw_2d(event, |_, g| { clear([0.0, 0.0, 0.0, 1.0], g); });
    }

    pub fn render_location<E>(
                       &self,
                       event : &E,
                       window : &mut PistonWindow,
                       client_resources: &mut ClientResources,
                       asset_manager: &mut AssetManager,
                       loc: &Location,
    ) where E : GenericEvent {
        self.render_location_objects(event, window, client_resources, asset_manager, loc);
        self.render_location_entities(event, window, client_resources, asset_manager, loc);
    }

    pub fn render_world<E>(
                       &self,
                       event : &E,
                       window : &mut PistonWindow,
                       client_resources: &mut ClientResources,
                       asset_manager: &mut AssetManager,
                       wid: WorldID,
                       hardcoded_assets : &HardcodedAssets,
                       longitude_center: f64,
    ) where E : GenericEvent {
        let planet_mask_tex = &hardcoded_assets.planet_mask;
        if let Ok(map_tex) = asset_manager.get_map_for(wid) {
            let normal_offset = -longitude_center*map_tex.get_size().0 as f64;
            let wrapped_offset = (if longitude_center > 0.5 {
                1.0-longitude_center
            } else {
                -1.0-longitude_center
            }) *map_tex.get_size().0 as f64;
            window.draw_2d(event, |c, g| {
                image(map_tex, c.transform.trans(WIDTH/2.0 + normal_offset , 50.0), g);
                image(map_tex, c.transform.trans(WIDTH/2.0 + wrapped_offset , 50.0), g);
                image(planet_mask_tex, c.transform.trans(0.0, 50.0), g);
            });
        }
    }
}

#[inline]
fn calc_zoom(sprite_pixels : u32, view_width_meters : f64, rendered_width_meters : f64) -> f64 {
    (WIDTH * rendered_width_meters)
    / (sprite_pixels as f64 * view_width_meters)
}

fn is_on_screen(sp : &CPoint2) -> bool {
    // true
    sp.x >= 0.0 && sp.x < WIDTH as f32
    && sp.y >= 0.0 && sp.y < HEIGHT as f32
}
