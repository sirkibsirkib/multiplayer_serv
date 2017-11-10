extern crate noise;
use std::mem;
use ::rand;
use super::game_state::Point;

use self::noise::{Seedable,NoiseModule,Perlin,Point2};

// perlins take up a lot of space. combine them for best results
pub struct NoiseMaster {
    perlins : [Perlin;10],
}

macro_rules! array_from_vec {
    ( $v:ident $($index:expr)+ ) => (
        [
            $(
                    $v.set_seed($index),
             )+
        ]
    );
}

impl NoiseMaster {
    pub fn new() -> NoiseMaster {
        let p1 = Perlin::new();
        NoiseMaster{
            perlins : array_from_vec![p1 0 1 2 3 4 5 6 7 8 9],
        }
    }

    pub fn generate_noise_field(&self, seeds : [usize;5], multipliers : [f32;5], zoom : f32) -> NoiseField {
        for s in seeds.iter() {
            assert!(*s < 5);
        }
        let norm_div : f32 = multipliers.iter().sum();
        assert!(norm_div > 0.0);
        NoiseField {
            zoom : zoom,
            perlins : [
                (&self.perlins[seeds[0]], multipliers[0]),
                (&self.perlins[seeds[1]], multipliers[1]),
                (&self.perlins[seeds[2]], multipliers[2]),
                (&self.perlins[seeds[3]], multipliers[3]),
                (&self.perlins[seeds[4]], multipliers[4]),
            ],
            norm_mult : 1.0 / norm_div,
        }
    }
}

pub struct NoiseField<'a> {
    zoom : f32,
    //higher means COARSER noise
    perlins : [(&'a Perlin,f32);5],
    norm_mult : f32, // sum of x.1 for all x in perlins
}

impl<'a> NoiseField<'a> {

    #[inline]
    fn pt_map(pt : Point, zoom : f32) -> [f32;2] {
        [
            pt[0] as f32 * zoom,
            pt[1] as f32 * zoom,
        ]
    }

    pub fn sample(&self, pt : Point) -> f32 {
        self.perlins
        .iter()
        .enumerate()
        .map(|(i, x)| self.norm_mult * x.1 * x.0.get(NoiseField::pt_map(pt,  (1.0 / (1 << i) as f32) * self.zoom)))
        //                             mult  perlin
        .fold(0.0, |x, y| x + y)
    }
}
