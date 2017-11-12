extern crate noise;
use self::noise::{Perlin,Seedable,NoiseModule};
use super::game_state::Point;
use ::identity::SuperSeed;

use ::rand::{SeedableRng,Rng,Isaac64Rng};

lazy_static! {
    static ref NUM_PERLINS : usize = 5;
    static ref PERLINS : [Perlin ; 5] = {
        let p1 = Perlin::new();
        [
            p1.set_seed(0),
            p1.set_seed(1),
            p1.set_seed(2),
            p1.set_seed(3),
            p1.set_seed(4),
        ]
    };
}

#[derive(Debug)]
pub struct NoiseField {
    perlins : [&'static Perlin ; NoiseField::PERLINS_PER_FIELD],
    mults : [f32 ; NoiseField::PERLINS_PER_FIELD],
    mults_sum : f32,
    super_seed : u64,
}

impl NoiseField {
    const PERLINS_PER_FIELD : usize = 3;

    pub fn get_super_seed(&self) -> SuperSeed {
        self.super_seed
    }

    pub fn from_super_seed(super_seed : SuperSeed) -> NoiseField {
        let mut rng = Isaac64Rng::from_seed(&[super_seed]);
        let mut mults : [f32;3];
        let mut mults_sum : f32;
        while {
            mults  = [
                rng.gen(),
                rng.gen(),
                rng.gen(),
            ];
            mults_sum = mults.iter().sum();
            //do-while...
            mults_sum == 0.0
        }{}
        NoiseField {
            perlins : [
                &PERLINS[rng.next_u32() as usize],
                &PERLINS[rng.next_u32() as usize],
                &PERLINS[rng.next_u32() as usize],
            ],
            mults : mults,
            mults_sum : mults_sum,
            super_seed : super_seed,
        }
    }

    #[inline]
    fn pt_map(pt : Point, zoom : f32) -> [f32;2] {
        [
            pt[0] as f32 * zoom,
            pt[1] as f32 * zoom,
        ]
    }

    pub fn sample(&self, pt : Point) -> f32 {
        let mut sample_tot : f32 = 0.0;
        let one : u32 = 1;
        for i in 0..Self::PERLINS_PER_FIELD {
            sample_tot +=
            (
                self.mults[i] * self.perlins[i].get(
                    Self::pt_map(
                        pt,
                        (1.0 / (self::ONE << i) as f32),
                    )
                )
            ) / self.mults_sum;
        };
        sample_tot
    }
}

const ONE : u32 = 1;
