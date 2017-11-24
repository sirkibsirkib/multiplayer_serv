use super::{Material};
use super::zones::{Zone};
use ::points::*;
use super::grid::TotalGrid;

const METERS_PER_ZONE_SAMPLE : i32 = 50;

pub struct LocationPrimitive {
    seed: u64,
    zone_in_world: Zone,
}

impl LocationPrimitive {
    pub fn new(seed: u64, zone_in_world: Zone) -> LocationPrimitive {
        LocationPrimitive {
            seed: seed,
            zone_in_world: zone_in_world,
        }
    }
}

pub struct Location {
    materials : TotalGrid<Material>,
}

impl Location {
    pub fn generate(loc_prim: LocationPrimitive) -> Location {
        let x_cells = METERS_PER_ZONE_SAMPLE
            * loc_prim.zone_in_world.get_samples_per_row();
        let y_cells = METERS_PER_ZONE_SAMPLE
            * loc_prim.zone_in_world.get_samples_per_col();
        let mut which_mat = |x, y| {
            Material::Water
        };
        Location {
            materials: TotalGrid::new_from_func(DPoint2::new(x_cells, y_cells), &mut which_mat),
        }
    }
}
