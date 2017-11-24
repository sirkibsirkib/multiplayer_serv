use ::std::path::{PathBuf,Path};

pub trait AppliesDiff<T> {
	fn apply_all(&mut self, t: &[T]){
		for x in t.iter() {self.apply_diff(x)}
	}
	fn apply_diff(&mut self, t : &T);
}

pub trait Primitive<T> : Sized {
	fn generate_new(self) -> T;
}

pub trait PrimitiveWithDiffs<T:AppliesDiff<D>,D> : Primitive<T> {
	fn generate_diffed(self, diffs: &Vec<D>) -> T {
		let mut x = self.generate_new();
		for d in diffs {
			x.apply_diff(d);
		}
		x
	}
}

//////////////////////////////////////////////

/*
EXAMPLE OF HOW TO USE:
1. You need {
    (A) finished product struct (eg World)
    (B) primitive which generates (A) (eg WorldPrimitive) and hopefully is much smaller than (A).
}

2. implement Primitive<(A)>::generate_new for (B)
    define how to generate a 'fresh' (A) given only the context of a (C)
//// YOU CAN STOP HERE! (Primitive) ready!

3. You need {
    (C) a struct/enum which encodes information of a single Diff on (A)
}

4. implement AppliesDiff::apply_diff for your (A) and (C)
    alter (A) in response to a (C) instance.

5. implement PrimitiveWithDiffs{} (no functions needed! defaults OK)


////////////////////////// EXAMPLE //////////////////////////////////
struct WorldPrimitive{init : i32}
enum WorldDiff {Add(i32), Sub(i32), Zeroify}
struct World {data : i32}

impl AppliesDiff<WorldDiff> for World {
	fn apply_diff(&mut self, t : &WorldDiff) {
		match t {
            &WorldDiff::Add(x) => self.data += x,
            &WorldDiff::Sub(x) => self.data -= x,
            &WorldDiff::Zeroify => self.data = 0,
        }
	}
}

impl Primitive<World> for WorldPrimitive {
	fn generate_new(&self) -> World {
		World{data: self.init}
	}
}
impl DiffedPrimitive<World,WorldDiff> for WorldPrimitive {}
*/
