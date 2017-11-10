extern crate noise;
use rand;

use self::noise::{Seedable,NoiseModule,Perlin,Point2};

struct NoiseField {
    coarseness : f64,
    perlins : [Perlin;3],
}

impl NoiseField {
    pub fn new() -> NoiseField() {
        mem::size_of::<i32>()
    }
}
