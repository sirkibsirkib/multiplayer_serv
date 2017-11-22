use ::std::path::{PathBuf,Path};

pub trait AppliesDiff<T> {
	fn apply_all(&mut self, t: &[T]){
		for x in t.iter() {self.apply_diff(x)}
	}
	fn apply_diff(&mut self, t : &T);
}

pub trait Primitive<T> {
	fn generate_new(&self) -> T;
}

pub trait PrimitiveWithDiffs<T:AppliesDiff<D>,D> : Primitive<T> {
	fn generate_diffed(&self, diffs: &Vec<D>) -> T {
		let mut x = self.generate_new();
		for d in diffs {
			x.apply_diff(d);
		}
		x
	}
}

pub trait SavableLoadable<'a> : PrimitiveWithDiffs<T,D> + Serializable + Deserializable<'a>
where D : Serializable + Deserializable<'a> {
    pub fn save_path(&self) -> &Path;
    pub fn get_prim(&self) -> P where P : Primitive<Self>;
    pub fn get_applied_diffs(&self) -> &Vec<D>;
    pub fn save(&self) {
        let mut prim_path = Path::new(&format!("{}_prim", self.save_path()));
        let mut diffs_path = Path::new(&format!("{}_diffs", self.save_path()));
        //save prim
        //save diff_vec
    }
    pub fn load() -> T {
        let mut prim_path = Path::new(&format!("{}_prim", self.save_path()));
        let mut diffs_path = Path::new(&format!("{}_diffs", self.save_path()));
        let prim = (); //TODO
        let diffs = ();
        prim.generate_diffed(diffs)
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
//// YOU CAN STOP HERE! (Primitive,PrimitiveWithDiffs) ready!

6. implement SavableLoadable::{save_path,get_prim,get_applied_diffs} for (A.)
    save_path -> an &Path of where to save/load (should be unique)
    get_prim -> returns a (C) which represents a starting point of the (A)
    get_applied_diffs -> return a vector of Diffs, representing steps from (C) to (A)
//// YOU CAN STOP HERE! (Primitive,PrimitiveWithDiffs,SavableLoadable) ready!


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
