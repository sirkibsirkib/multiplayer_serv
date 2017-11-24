use ::std::hash;
use ::std::mem;
use ::std::ops::{Add,Sub};

//continuous point in 2d space
#[derive(Copy,Clone,Debug,Serialize,Deserialize)]
pub struct CPoint2 {
    pub x: f32,
    pub y: f32,
}

impl CPoint2 {
    pub fn new(x: f32, y: f32) -> CPoint2 {
        CPoint2{x: x, y: y}
    }
    pub fn discrete(self) -> DPoint2 {
        DPoint2::new(self.x as i32, self.y as i32)
    }

    pub fn div_scale(self, denominator: f32) -> Self {
        CPoint2::new(self.x / denominator, self.y / denominator)
    }

    pub fn scale(self, scale: f32) -> Self {
        CPoint2::new(self.x * scale, self.y * scale)
    }

    pub fn dist_to(self, o: Self) -> f32 {
        let dx = self.x - o.x;
        let dy = self.y - o.y;
        dx.hypot(dy)
    }

    pub fn skewed_dist_to(self, o: Self, mult_x: f32, mult_y: f32) -> f32 {
        let dx = self.x - o.x;
        let dy = self.y - o.y;
        (dx*dx*mult_x + dy*dy*mult_y).sqrt()
    }
}

impl hash::Hash for CPoint2 {
    fn hash<H>(&self, state: &mut H)
    where H: hash::Hasher {
        assert!(!self.x.is_nan());
        let tx : u32 =  unsafe { mem::transmute(self.x) };
        tx.hash(state);
        assert!(!self.y.is_nan());
        let ty : u32 = unsafe { mem::transmute(self.y) };
        ty.hash(state);
    }
}

impl PartialEq for CPoint2 {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
impl Eq for CPoint2 {}

impl Add for CPoint2 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for CPoint2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

//continuous point in 3d space
#[derive(Copy,Clone,Debug,Serialize,Deserialize)]
pub struct CPoint3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl CPoint3 {

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self{x: x, y: y, z: z}
    }
    pub fn scale(mut self, scale: f32) {
        self.x *= scale;
        self.y *= scale;
        self.z *= scale;
    }
}

//discrete point in 2d space
#[derive(Copy,Clone,Debug,PartialEq,Hash,Eq,Serialize,Deserialize)]
pub struct DPoint2 {
    pub x: i32,
    pub y: i32,
}

impl DPoint2 {
    pub const NULL : Self = DPoint2{x: 0, y: 0};
    pub fn shift_x(self, shift: i32) -> DPoint2 {
        DPoint2{x: self.x+shift, y: self.y}
    }
    pub fn shift_y(self, shift: i32) -> DPoint2 {
        DPoint2{x: self.x, y: self.y+shift}
    }
    pub fn new(x: i32, y: i32) -> DPoint2 {
        DPoint2{x: x, y: y}
    }
    pub fn continuous(self) -> CPoint2 {
        CPoint2::new(self.x as f32, self.y as f32)
    }
}

impl Add for DPoint2 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for DPoint2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}
