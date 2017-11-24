use ::points::*;

struct QuadTree<T> {
    branches: QuadOpt<T>,
    tl: CPoint2,
    br: CPoint2,
}

enum QuadOpt<T> {
    Zero,
    One(T),
    Four(Box<[QuadOpt<T>; 4]>),
}

impl<T> QuadTree<T> {
    pub fn new(tl: CPoint2, br: CPoint2) -> Self {
        QuadTree {
            branches: QuadOpt::Zero,
            tl: tl,
            br: br,
        }
    }
    //TODO
}
