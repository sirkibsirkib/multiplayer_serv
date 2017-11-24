use ::points::DPoint2;

pub struct TotalGrid<T> {
    elements: Vec<T>,
    width: i32,
}

fn cell_from_index(width: i32, index: i32) -> DPoint2 {
    DPoint2::new(index % width, index / width)
}

impl<'a, T> IntoIterator for &'a TotalGrid<T> {
    type Item = (DPoint2, &'a T);
    type IntoIter = TotalGridIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        TotalGridIterator {
            elements: &self.elements[..],
            next_element: 0,
            width: self.width,
        }
    }
}

pub struct TotalGridIterator<'a, T: 'a> {
    elements: &'a [T],
    next_element: usize,
    width: i32,
}

impl<'a, T> Iterator for TotalGridIterator<'a, T> {
    type Item = (DPoint2,&'a T);

    fn next(&mut self) -> Option<(DPoint2,&'a T)> {
        let old = self.next_element;
        self.next_element += 1;
        if old == self.elements.len()-1 {
            None
        } else {
            Some(
                (
                    cell_from_index(self.width, self.next_element as i32),
                    self.elements.get(self.next_element).expect("FUARK"),
                )
            )
        }
    }
}

impl<T> TotalGrid<T> {
    fn vec_elements(&self) -> i32 {
        self.elements.len() as i32
    }

    // pub const TRIVIAL : TotalGrid<T> = TotalGrid{elements: Vec::new(), dimensions: DPoint2::NULL};

    pub fn cell_iterator<'a>(&'a self) -> Box<Iterator<Item=DPoint2> + 'a> {
        Box::new(
            (0..self.vec_elements())
            .map(move |i| cell_from_index(self.width, i))
        )
    }
    pub fn get_width(&self) -> i32 {self.width}
    pub fn get_height(&self) -> i32 {self.elements.len() as i32 / self.width}
    pub fn get_dimensions(&self) -> DPoint2 {DPoint2{x: self.get_width(), y:self.get_height()}}

    // fn cell_from_index(&self, index: i32) -> DPoint2 {
    //     DPoint2::new(index % self.dimensions.x, index / self.dimensions.x)
    // }

    pub fn new_from_func(dimensions: DPoint2, f: &mut FnMut(usize, usize) -> T) -> TotalGrid<T>{
        let mut v = Vec::with_capacity((dimensions.x*dimensions.y) as usize);
        for y in 0..dimensions.y {
            for x in 0..dimensions.x {
                v.push(f(x as usize, y as usize));
            }
        }
        TotalGrid {
            elements: v,
            width: dimensions.x,
        }
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut T {
        &mut self.elements[x*y]
    }

    pub fn get(&self, x: usize, y: usize) -> &T {
        &self.elements[x*y]
    }

    pub fn maybe_get(&self, x: usize, y: usize) -> Option<&T> {
        self.elements.get(x*y)
    }

    pub fn maybe_get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.elements.get_mut(x*y)
    }

    pub fn put(&mut self, x: usize, y: usize, element: T) {
        self.elements[x*y] = element;
    }
}

pub struct TotalGridBuilder<T> {
    elements: Vec<T>,
}

impl<T> TotalGridBuilder<T> {
    pub fn new() -> TotalGridBuilder<T> {
        TotalGridBuilder {
            elements: vec![],
        }
    }

    pub fn append(&mut self, element: T) {
        self.elements.push(element);
    }

    pub fn replace(&mut self, element: T) {
        self.elements.push(element);
    }

    pub fn ready(&self, dimensions: DPoint2) -> bool {
        dimensions.x*dimensions.y == self.elements.len() as i32
    }

    pub fn finalize(self, dimensions: DPoint2) -> Result<TotalGrid<T>, ()> {
        if self.ready(dimensions) {
            Ok(TotalGrid {
                elements: self.elements,
                width: dimensions.x,
            })
        } else {
            Err(())
        }
    }
}
