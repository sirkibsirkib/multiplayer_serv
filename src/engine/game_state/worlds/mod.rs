use ::points::*;
use ::rand::{SeedableRng,Rng,Isaac64Rng};
use std::path::Path;

pub mod zones;
use ::utils::*;
use ::utils::noise::*;
use ::utils::funcs::*;
use ::identity::*;


lazy_static! {
    pub static ref START_WORLD_PRIM : WorldPrimitive = WorldPrimitive::new(0, 0.4);
    pub static ref START_WORLD : World = World::new(*START_WORLD_PRIM);
}

// use super::portals::UniquePoint;
// use self::location::LocationPrimitive;
use self::zones::{Zone,WorldLink};

extern crate image;

const AZIMUTH_SHIFT: f32 = 0.006;
const AZIMUTH_MULT: f32 = 1.0 - AZIMUTH_SHIFT;

type FloatPixel = [f32 ; 3];
type U8Pixel = [u8 ; 3];

pub struct NoiseSample {
    pub base: f32,
    pub complex: f32,
    pub temp: f32,
}

#[derive(Debug,Copy,Clone)]
pub struct PointSampleData {
    pub temp: f32,
    pub height: f32,
    pub x_slope: f32,
    pub y_slope: f32,
    pub slope: f32,
}

#[derive(PartialEq,Eq,Copy,Clone,Debug)]
pub enum Material {
    Rock, Trees, Grass, Water, Ice, Snow, DarkRock, Sand,
}

impl Material {
    fn is_land(self) -> bool {
        !(self == Material::Water)
    }
    fn col(&self) -> FloatPixel {
        match self {
            &Material::Rock => [0.5, 0.37, 0.24],
            &Material::DarkRock => [0.41, 0.27, 0.21],
            &Material::Trees => [0.3, 0.55, 0.45],
            &Material::Grass => [0.6, 0.75, 0.4],
            &Material::Water => [0.3, 0.5, 1.0],
            &Material::Ice => [1.0, 1.0, 1.3],
            &Material::Snow => [1.2, 1.2, 1.25],
            &Material::Sand => [1.0, 0.95, 0.8],
        }
    }
}


fn px_bleach(x: FloatPixel, bleaching: f32) -> FloatPixel {
    [
        x[0] + ((1.0-x[0]) * bleaching),
        x[1] + ((1.0-x[1]) * bleaching),
        x[2] + ((1.0-x[2]) * bleaching),
    ]
}

fn px_shade(x: FloatPixel, shading: f32) -> FloatPixel {
    [
        x[0] * (1.0 - shading),
        x[1] * (1.0 - shading),
        x[2] * (1.0 - shading),
    ]
}

fn px_bound(x: f32) -> f32 {
    if x < 0.0 {0.0}
    else if x > 1.0 {1.0}
    else {x}
}

fn px_finalize(x: FloatPixel) -> U8Pixel {
    [(px_bound(x[0]) * 254.0) as u8,
    (px_bound(x[1]) * 254.0) as u8,
    (px_bound(x[2]) * 254.0) as u8]
}

#[derive(Copy,Clone,Debug,Serialize,Deserialize)]
pub struct WorldPrimitive {
    super_seed: u64,
    star_energy: f32,
}

impl WorldPrimitive {
    pub fn new(super_seed: u64, star_energy: f32) -> WorldPrimitive {
        WorldPrimitive {
            super_seed: super_seed,
            star_energy: star_energy,
        }
    }
}

enum Weighting {
    Equal, Higher(f32), Lower(f32),
}


#[derive(Debug)]
pub struct World {
    base_height: NoiseField,
    complex_height: NoiseField,
    temp_nf: NoiseField,
    water_level: f32,
    grass_within: [f32;2],
    trees_within: [f32;2],
    size: f32,
    zones: Vec<Zone>,
    links: Vec<WorldLink>,
    wp: WorldPrimitive,
    exit_points: Vec<UniquePoint>,
}


impl World {

    pub fn get_zone(&self, zone_id: usize) -> &Zone {
        &self.zones[zone_id]
    }

    pub fn get_size(&self) -> f32 {self.size}

    fn gen_between<R:Rng>(lower: f32, upper: f32, w: Weighting, rng: &mut R) -> f32 {
        let betweenyness = match w {
            self::Weighting::Equal => {
                rng.gen::<f32>()
            },
            self::Weighting::Lower(amp) => {
                1.0 - sigmoid(rng.gen::<f32>(), amp)
            },
            self::Weighting::Higher(amp) => {
                sigmoid(rng.gen::<f32>(), amp)
            },
        };
        lower * betweenyness
        + upper * (1.0 - betweenyness)
    }

    pub fn new<'b>(wp: WorldPrimitive) -> World {
        let mut rng = Isaac64Rng::from_seed(&[wp.super_seed]);

        let size = 0.2 + rng.gen::<f32>()*0.25*(1.0-wp.star_energy);
        let water_level = sig_0_pt5(rng.gen::<f32>() - 0.5 + (size * (1.0 - wp.star_energy)), 2.02);
        let base_height_bounds = [
            Self::gen_between(0.05, 0.15, Weighting::Lower(3.0), &mut rng),
            Self::gen_between(0.15, 1.4, Weighting::Lower(2.0), &mut rng),
        ];

        let base_height = NoiseField::generate(&mut rng, base_height_bounds, 4)
        .agglomerate(
            NoiseField::generate(&mut rng, [30.2, 50.4], 2), Some((1.0, 0.013))
        ).agglomerate(
            NoiseField::generate(&mut rng, [2.2, 9.4], 3), Some((1.0, 0.1))
        );

        let complex_height_bounds = [
            Self::gen_between(0.06, 0.4, Weighting::Lower(2.0), &mut rng),
            Self::gen_between(2.5, 5.0, Weighting::Lower(1.2), &mut rng),
        ];

        let complex_height = NoiseField::generate(&mut rng, complex_height_bounds, 4)
        .agglomerate(
            NoiseField::generate(&mut rng, [15.2, 32.4], 3), Some((1.0, 0.12))
        );

        let temp_nf = NoiseField::generate(&mut rng, [30.0, 100.0], 3).agglomerate(
            NoiseField::generate(&mut rng, [0.8, 6.0], 2), Some((1.0, 0.25))
        );

        let mut w = World {
            size: size,
            base_height: base_height,
            complex_height: complex_height,
            temp_nf: temp_nf,
            water_level: water_level,
            grass_within: [0.1,0.2],
            trees_within: [0.1,0.3],
            zones: Vec::new(),
            links: Vec::new(),
            exit_points: Vec::new(),
            wp: wp,
        };
        w.zones = zones::generate_zones_for(&w, &mut rng);
        w.links = zones::generate_links_for(&w.zones, &mut rng, &w);
        println!("{} zones, {} links", w.zones.len(), w.links.len());
        w
    }

    fn calc_temp_at(&self, pt: CPoint2, height: f32) -> f32 {
        let x = self.temp_nf.sample_3d(equirectangular(pt)) * 0.5 + 0.5;
        x * 0.15
        + (1.0-height)
        - sigmoid(self.size / (Self::pole_distance(pt.y) + 0.01), 1.0) * 0.7
        + self.wp.star_energy
        - 0.5
    }

    fn material_at(&self, pt: CPoint2, point_data: &PointSampleData) -> Material {
        let veg_dist = (((point_data.temp - 0.3).abs() + 0.01) * (point_data.slope*20.0 + point_data.height)).abs();
        if point_data.height < self.water_level {
            if point_data.temp < -0.05 {Material::Ice}
            else {Material::Water}
        }
        else if point_data.temp < -0.3+self.water_level {Material::Snow}
        else if point_data.slope > 0.1+(0.13 * self.wp.star_energy) && ((point_data.temp * 3.0) as i8 %3!=0) {Material::DarkRock}
        else if (point_data.temp-0.45).abs() + 0.6*(self.size-0.7).abs() < 0.28 && ((point_data.temp * 14.0) as i8 %3==0)  {
            Material::Trees
        }

        else if (point_data.height-self.water_level).abs()*(point_data.slope + 0.001) < 0.018
        && (point_data.height * 6.0) as i8 %2==0 && point_data.x_slope < point_data.y_slope*1.6 {
            Material::Sand
        }
        else if (point_data.temp-0.52).abs() + (point_data.height-self.water_level).abs() < 0.3 && ((point_data.temp * 6.0) as i8 %3!=0)  {
            Material::Grass
        }
        else {Material::Rock}
    }

    // (0.0, 1.0)
    fn calc_height_at(&self, pt: CPoint2) -> f32 {
        let rough_sample = (self.base_height.sample_3d(equirectangular(pt)) * 0.5 + 0.5).powf(1.55);
        if rough_sample > self.water_level {
            let fine_sample = self.complex_height.sample_3d(equirectangular(pt)) * 0.5 + 0.5;
            let fineness = rough_sample - self.water_level;
            fineness * fine_sample + (1.0 - fineness) * rough_sample
        } else {
            rough_sample
        }
    }

    fn pole_distance(y: f32) -> f32 {
        assert!(y <= 1.0 && y >= 0.0);
        if y < 0.5 {y} else {1.0-y}
    }

    fn calc_sample_data_at(&self, pt: CPoint2) -> PointSampleData {
        let height = self.calc_height_at(pt);
        let x_slope = {
            sigmoid(
                (height - self.calc_height_at(CPoint2::new((pt.x+AZIMUTH_SHIFT % 1.0), pt.y))) * 0.5,
                30.0,
            )
        };
        let y_slope = {
            sigmoid(
                if pt.y < AZIMUTH_MULT {
                    (height - self.calc_height_at(CPoint2::new(pt.x, pt.y+AZIMUTH_SHIFT))) * 0.5
                } else {
                    (self.calc_height_at(CPoint2::new(pt.x, pt.y-AZIMUTH_SHIFT)) - height) * 0.5
                },
                30.0,
            )
        };
        let slope = (x_slope.abs() + y_slope.abs()) * 0.5;

        let temp = self.calc_temp_at(pt, height);
        PointSampleData {
            height: height,
            x_slope: x_slope,
            y_slope: y_slope,
            slope: slope,
            temp: temp,
        }
    }

    fn zoned_pixel_sample(&self, pt: CPoint2) -> U8Pixel {
        for l in self.links.iter() {
            if point_is_wider_roughly_between(l.get_world_a_pt(), pt, l.get_world_b_pt()) {
                return px_finalize(px_bleach({
                    if pt_wider_dist(l.get_world_a_pt(), pt) < pt_wider_dist(l.get_world_b_pt(), pt)
                    {l.get_mat_a()} else {l.get_mat_b()}
                }.col(), 0.3));
            }
        }
        for (k, v) in self.zones.iter().enumerate() {
            if v.barely_within(pt) {
                return [255,  (k as u8*21 + 200), (k as u8*31)];
            }
            if v.close_to_cell(pt) {
                return [255,  0, 0];
            }
        }
        self.pixel_sample(pt)
    }

    fn pixel_sample(&self, pt: CPoint2) -> U8Pixel {
        let point_data = self.calc_sample_data_at(pt);
        px_finalize(
            {
                let mat = self.material_at(pt, &point_data);
                if mat == Material::Water || mat == Material::Ice {
                    px_shade(mat.col(), 0.25 + ((1.0-point_data.height) * 0.4))
                } else {
                    px_shade(mat.col(), 0.25 + ((point_data.x_slope*0.5 + 0.5) * 0.4))
                }
            }
        )
    }


    pub fn to_png(&self, path: &Path, pix_height: u32) -> Result<(), ::std::io::Error>{
        let pix_width = pix_height * 2;
        let (f_width, f_height) = (pix_width as f32, pix_height as f32);
        let mut pixels: Vec<u8> = vec![];
        let time_0 = ::std::time::Instant::now();
        for y in 0..pix_height {
            for x in 0..pix_width {
                for u_eight in self.pixel_sample(
                    CPoint2::new(x as f32 / f_width, y as f32 / f_height)
                ).into_iter() {
                    pixels.push(*u_eight);
                }
                pixels.push(255); //a
            }
        }
        let time_1 = ::std::time::Instant::now();
        let res = image::save_buffer(path, &pixels[..], pix_width, pix_height, image::RGBA(8));
        let time_2 = ::std::time::Instant::now();
        // let (dur_0, dur_1) = (time_1-time_0, time_2-time_1);
        res
    }
}

fn pt_wider_dist(a: CPoint2, b: CPoint2) -> f32 {
    a.skewed_dist_to(b, 2.0, 1.0)
}

// fn pt_dist(a: CPoint2, b: CPoint2) -> f32 {
//     let x = a[0] - b[0];
//     let y = a[1] - b[1];
//     (x*x + y*y).powf(0.5)
// }

// Return whether b is between a and c, allowing for distance epsilon
fn point_is_wider_roughly_between(a: CPoint2, b: CPoint2, c: CPoint2) -> bool {
    // use ::std::cmp::{max};
    let ab = pt_wider_dist(a,b);
    let bc = pt_wider_dist(b,c);
    let zoop = if ab < bc {ab} else {bc};
    let zoop = if zoop < 0.0000001 {0.0000001} else {zoop};
    ab+bc <= pt_wider_dist(a,c) + (0.000001) / zoop.powi(2)
    //epsilon/100000.0/max_three(sigmoid(ab*100.0, 1.9),sigmoid(bc*100.0, 1.9),0.0000001)
}

const HHH: f32 = 1.0;
pub fn equirectangular(pt: CPoint2) -> CPoint3 {
    let (x, y) = (pt.x, pt.y);
    let latitude_radius = (1.0 - (y*2.0 - 1.0).abs()).powf(0.7);
    let l = x * ::std::f32::consts::PI * 2.0;
    CPoint3::new(
        l.sin() * latitude_radius * HHH,
        l.cos() * latitude_radius * HHH,
        y * ::std::f32::consts::PI, //PI instead of 2PI to make it 2* wider
    )
}
