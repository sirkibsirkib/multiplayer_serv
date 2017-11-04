
use std::iter::IntoIterator;
use super::game_state::Point;


pub struct QuadTree<T:HasPoint> {
    contents : QuadEnum<T>,
    tl : Point,
    width : f64,
}


enum QuadEnum<T:HasPoint> {
    Many(Box<[QuadEnum<T>;4]>),
    One(T),
    Nothing,
}

/*
ONE QUADTREE. this quadtree then just contains enums all the way down
*/

pub trait HasPoint {
    fn point(&self) -> Point;
}

impl<T:HasPoint> QuadTree<T> {
    pub fn new_from<Q : HasPoint>(items : Vec<Q>, tl : Point, width : f64) -> QuadTree<Q> {
        unimplemented!()
    }

    pub fn new_empty<Q : HasPoint>(tl : Point, width : f64) -> QuadTree<Q> {
        QuadTree {
            contents : QuadEnum::Nothing,
            tl : tl,
            width : width,
        }
    }

    pub fn is_within(&self, x : &T) -> bool {
        unimplemented!()
    }

    pub fn add(&mut self) {
        unimplemented!();
        //recurse deeper, splitting if necessary
    }
}
