extern crate noise;
use std::mem;
use ::rand;
use super::game_state::Point;
use ::identity::SuperSeed;

use self::noise::{Seedable,NoiseModule,Perlin,Point2};

// perlins take up a lot of space. combine them for best results

//TODO make noisemaster handle it
#[derive(Debug)]
pub struct NoiseMaster {
    // perlins : [Perlin ; NoiseMaster::PERLINS],
}
//
// macro_rules! set_seed_arr {
//     ( $v:ident $($index:expr)+ ) => (
//         [
//             $(
//                     $v.set_seed($index),
//              )+
//         ]
//     );
// }

impl NoiseMaster {
    pub fn new() -> NoiseMaster {
        NoiseMaster{}
    }
    // const PERLINS : usize = 10;
    //
    // pub fn new() -> NoiseMaster {
    //     let p1 = Perlin::new();
    //     NoiseMaster{
    //         perlins : set_seed_arr![p1 0 1 2 3 4 5 6 7 8 9],
    //     }
    // }
    //
    // fn give_perlin(&self, seed : u8) -> &'static Perlin {
    //     &'static self.perlins[((seed as usize) % Self::PERLINS)]
    // }
}


#[derive(Debug)]
pub struct NoiseField {
    // seeds -> perlins for quicker access
    // perlins : [Perlin ; NoiseField::PERLINS_PER_FIELD],
    // norm_mult : f32, // sum of x.1 for all x in perlins
    super_seed : u64,
}

impl NoiseField {


    const PERLINS_PER_FIELD : usize = 3;

    pub fn get_super_seed(&self) -> SuperSeed {
        self.super_seed
    }

    pub fn from_super_seed(super_seed : SuperSeed, noise_src : &NoiseMaster) -> NoiseField {
        // let norm_div : f32 = nf_key.multipliers.iter().sum();
        // assert!(norm_div > 0.0);
        // let p1 = Perlin::new();
        NoiseField {
            // perlins : [
            //     p1.set_seed(nf_key.seeds[0] as u32),
            //     p1.set_seed(nf_key.seeds[1] as u32),
            //     p1.set_seed(nf_key.seeds[2] as u32),
            // ],
            // norm_mult : 1.0 / norm_div,
            super_seed : super_seed,
        }
    }

    // #[inline]
    // fn pt_map(pt : Point, zoom : f32) -> [f32;2] {
    //     [
    //         pt[0] as f32 * zoom,
    //         pt[1] as f32 * zoom,
    //     ]
    // }

    pub fn sample(&self, pt : Point) -> f32 {
        0.0
        // let mut sample_tot : f32 = 0.0;
        // for i in 0..Self::PERLINS_PER_FIELD {
        //     sample_tot +=
        //         self.norm_mult
        //         * self.nf_key.multipliers[i]
        //         * self.perlins[i].get(
        //             Self::pt_map(
        //                 pt,
        //                 (1.0 / (1 << i) as f32) * self.nf_key.zoom,
        //             )
        //         );
        // };
        // sample_tot
    }
}
