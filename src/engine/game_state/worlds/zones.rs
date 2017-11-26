
use super::{Material,PointSampleData,World,pt_wider_dist};
use super::grid::{TotalGrid,TotalGridBuilder};
use ::points::*;
use::rand::{Rng};
use std::collections::{HashMap};

#[derive(Debug,Copy,Clone)]
pub struct ZoneSample {
    pt: CPoint2,
    data: PointSampleData,
    mat: Material,
}

#[derive(Clone)]
pub struct Zone {
    tl: CPoint2,
    br: CPoint2,
    samples: TotalGrid<ZoneSample>,
}


fn traversible_link(zones: &Vec<Zone>, a: CPoint2, b: CPoint2, by_land: bool, w: &World) -> bool {
    let pt_checks : u8 = (pt_wider_dist(a, b)*30.0) as u8;
    for c in 0..pt_checks {
        let a_ratio = (c+1) as f32 / (pt_checks + 1) as f32;
        let b_ratio = 1.0 - a_ratio;
        let checked_pt = a.scale(a_ratio) + b.scale(b_ratio);

        let mat = w.material_at(checked_pt, &w.calc_sample_data_at(checked_pt));
        if by_land != mat.is_land() {
            // water in way / land in way
            return false;
        }
        for z in zones.iter() {
            if z.within(checked_pt){
                //The link crosses some other zone!
                return false;
            }
        }
    }
    true
}

impl Zone {
    pub fn get_samples_per_row(&self) -> i32 {
        self.samples.get_width()
    }

    pub fn get_samples_per_col(&self) -> i32 {
        self.samples.get_height()
    }

    pub fn close_to_cell(&self, pt: CPoint2) -> bool {
        for (k, v) in (self.samples).into_iter() {
            if v.pt.skewed_dist_to(pt, 2.0, 1.0) < 0.003 {
                return true
            }
        }
        false
    }

    pub fn coord_is_left(&self, c : DPoint2) -> bool {c.x == 0}
    pub fn coord_is_right(&self, c : DPoint2) -> bool {c.x == (self.samples.get_width() as i32)-1}
    pub fn coord_is_top(&self, c : DPoint2) -> bool {c.y == 0}
    pub fn coord_is_bottom(&self, c : DPoint2) -> bool {c.y == (self.samples.get_height() as i32)-1}

    pub fn coord_is_edge(&self, c : DPoint2) -> bool {
        self.coord_is_left(c)
        || self.coord_is_right(c)
        || self.coord_is_top(c)
        || self.coord_is_bottom(c)
    }

    pub fn coord_is_corner(&self, c : DPoint2) -> bool {
        (self.coord_is_left(c) || self.coord_is_right(c))
        && (self.coord_is_top(c) || self.coord_is_bottom(c))
    }

    fn boundary_sample_iter<'a>(&'a self) -> Box<Iterator<Item=(DPoint2,&ZoneSample)> + 'a> {
        Box::new(
            (& self.samples).into_iter()
            .filter(move |k_v| {
                self.coord_is_edge(k_v.0)
            }).map(|k_v| (k_v.0, k_v.1))
        )
    }

    fn shortest_sample_link(
        &self,
        self_taken: &[DPoint2],
        other: &Zone,
        other_taken: &[DPoint2],
        zones: &Vec<Zone>,
        w: &World,
    ) -> Option<WorldLink> {
        let mut shortest: Option<WorldLink> = None;
        for (m_coord, my_sample) in self.boundary_sample_iter() {
            if inside_slice(&m_coord, self_taken) || self.coord_is_corner(m_coord) {continue}
            for (t_coord, their_sample) in other.boundary_sample_iter() {
                if inside_slice(&t_coord, other_taken) || other.coord_is_corner(t_coord) {continue}
                let dist = pt_wider_dist(my_sample.pt, their_sample.pt);

                //only allow links where the material allows for it
                if my_sample.mat.is_land() != their_sample.mat.is_land()
                || dist > WorldLink::MAX_LEN
                || ! traversible_link(zones,  my_sample.pt, their_sample.pt, my_sample.mat.is_land(), w) {
                    continue;
                }

                if let Some(ref previous) = shortest {
                    if previous.length() < dist {continue}
                }
                shortest = Some(WorldLink {
                    zone_a_coord: m_coord,
                    zone_b_coord: t_coord,
                    world_a_pt: my_sample.pt,
                    world_b_pt: their_sample.pt,
                    mat_a: my_sample.mat,
                    mat_b: their_sample.mat,
                    land_link: their_sample.mat.is_land(),
                });
            }
        }
        shortest
    }

    pub fn new(tl: CPoint2, br: CPoint2, samples: TotalGrid<ZoneSample>) -> Zone {
        Zone {
            tl: tl,
            br: br,
            samples: samples,
        }
    }

    pub fn barely_within(&self, pt: CPoint2) -> bool {
        //constructs another bogus zone on the spot which is just a smaller zone inside
        let inner_tl = self.tl.scale(0.95) + self.br.scale(0.05);
        let inner_br = self.tl.scale(0.05) + self.br.scale(0.95);
        let inner_within = {
            (inner_tl.x <= pt.x && pt.x <= inner_br.x)
            && (inner_tl.y <= pt.y && pt.y <= inner_br.y)
        };
        self.within(pt) && !inner_within
    }

    pub fn within(&self, other: CPoint2) -> bool {

        (self.tl.x <= other.x && other.x <= self.br.x)
        && (self.tl.y <= other.y && other.y <= self.br.y)
    }

    pub fn overlaps_with(&self, other: (CPoint2,CPoint2)) -> bool { //other.0 == other.tl
        ! {
            //my left is right of your right
            self.tl.x > other.1.x
            //my right is left of your left
            || self.br.x < other.0.x
            //my top is below your bottom
            || self.tl.y > other.1.y
            //my bottom is above your top
            || self.br.y < other.0.y

        }
    }
}
// use::std::io::write;
impl::std::fmt::Debug for Zone {
    fn fmt(&self, f: &mut::std::fmt::Formatter) -> Result<(),::std::fmt::Error> {
        write!(f, "tl:{:?}, br:{:?}, ({},{})",
            self.tl, self.br, self.get_samples_per_row(), self.get_samples_per_col(),
        )
    }
}

pub fn generate_zones_for<R:Rng>(w: &World, rng: &mut R) -> Vec<Zone> {
    let distance_per_x_step = w.get_size() * 0.06;
    let distance_per_y_step = distance_per_x_step * 2.0;

    let mut stop_when_zero: i16 = 500;
    let mut zones: Vec<Zone> = vec![];
    'outer: while stop_when_zero > 0 {
        let tl = CPoint2::new(
            rng.gen::<f32>()*0.9 + 0.05,
            rng.gen::<f32>()*0.9 + 0.05,
        );
        let zone_sample_dim = DPoint2::new(
            (rng.gen::<u8>() % 7) as i32 + 5,
            (rng.gen::<u8>() % 7) as i32 + 5,
        );
        // (steps-1) to make sure the last sample points are ON the right/bottom edges
        let br = CPoint2::new(
            tl.x + (zone_sample_dim.x-1) as f32 * distance_per_x_step,
            tl.y + (zone_sample_dim.y-1) as f32 * distance_per_y_step,
        );
        if br.x > 0.95 || br.y > 0.95 {
            stop_when_zero -= 1;
            continue 'outer;
        }
        for z in zones.iter() {
            if z.overlaps_with((tl,br)){
                //OVERLAP
                stop_when_zero -= 7;
                continue 'outer;
            }
        }
        let mut samples: TotalGridBuilder<_> = TotalGridBuilder::new();
        let mut count_walkable_materials = 0;
        for y in 0..zone_sample_dim.y {
            for x in 0..zone_sample_dim.x {
                // let coord = DPoint2::new(x as i32, y as i32);
                let offset = CPoint2::new(x as f32 * distance_per_x_step, y as f32 * distance_per_y_step);
                // let (x_offset, y_offset) = (x as f32 * distance_per_x_step, y as f32 * distance_per_y_step);
                let pt = tl + offset;

                            assert!(pt.y <= 1.0);
                let point_data = w.calc_sample_data_at(pt);
                let mat = w.material_at(pt, &point_data);
                match mat {
                    Material::Water | Material::Ice => (),
                    _ => count_walkable_materials += 1,
                }
                samples.append(ZoneSample{data:point_data, pt:pt, mat:mat});
            }
        }
        if count_walkable_materials >= 5 {
            // not too much ice|water
            zones.push(
                Zone::new(tl, br, samples.finalize(zone_sample_dim).unwrap())
            );
            stop_when_zero -= zones.len() as i16 + 1; //prevent overload
        } else {
            stop_when_zero -= 2;
        }
        if zones.len() >= 3 {stop_when_zero -= 2}
        if zones.len() >= 5 {stop_when_zero -= 4}
        if zones.len() >= 7 {stop_when_zero -= 10}
    }
    zones
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub struct WorldLink {
    zone_a_coord: DPoint2,
    zone_b_coord: DPoint2,
    world_a_pt: CPoint2,
    world_b_pt: CPoint2,
    mat_a: Material,
    mat_b: Material,
    land_link: bool,
}


fn inside_slice<T: PartialEq>(x: &T, slice: &[T]) -> bool {
    for q in slice.iter() {
        if x==q {return true}
    }
    false
}

impl WorldLink {
    const MAX_LEN : f32 = 0.17;
    pub fn length(&self) -> f32 {
        pt_wider_dist(self.world_a_pt, self.world_b_pt)
    }
    pub fn get_world_a_pt(&self) -> CPoint2 {self.world_a_pt}
    pub fn get_world_b_pt(&self) -> CPoint2 {self.world_b_pt}
    pub fn get_mat_a(&self) -> Material{self.mat_a}
    pub fn get_mat_b(&self) -> Material{self.mat_b}
}

pub fn generate_links_for<R:Rng>(zones: &Vec<Zone>, rng: &mut R, w : &World) -> Vec<WorldLink> {
    let mut z : Vec<_> = zones.iter().collect();
    rng.shuffle(&mut z);
    let mut links: Vec<WorldLink> = vec![];
    let mut taken_samples: Vec<Vec<DPoint2>> = zones.iter().map(|_| vec![]).collect();
    for (i, zone_i) in zones.iter().enumerate() {
        'pair_loop: for (j, zone_j) in zones.iter().enumerate().skip(i+1) {
            if let Some(shortest) = zone_i.shortest_sample_link(
                &taken_samples[i], zone_j, &taken_samples[j], zones, w
            ) {
                if rng.gen_weighted_bool(6) {
                    // ignore connections randomly
                    continue
                }
                if shortest.length() <= WorldLink::MAX_LEN {
                    taken_samples[i].push(shortest.zone_a_coord);
                    taken_samples[j].push(shortest.zone_b_coord);
                    links.push(shortest);
                }
            }
        }
    }
    links
}
