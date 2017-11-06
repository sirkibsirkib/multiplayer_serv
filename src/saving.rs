use std::io;
use std::io::prelude::*;
use std::fs::File;

use serde::Serialize;
use serde::de::DeserializeOwned;
use bincode;
// use serde_json;
use std::path::{Path,PathBuf};
use std::io::{ErrorKind,Error};
use std::fs::create_dir;
use std::fmt::Debug;


pub struct SaverLoader {
    save_dir : Box<PathBuf>,
}

impl SaverLoader {
    pub fn new(save_dir : &str) -> SaverLoader {
        let p = Path::new(save_dir);
        if ! Path::new(p).exists() {
            println!("CREATING NEW DIR");
            create_dir(Path::new(p)).expect("Couldn't create new save dir");
        }
        SaverLoader {
            save_dir : Box::new(p.to_path_buf())
        }
    }

    pub fn save_me<X>(&self, x : &X, file : &str) -> Result<(), io::Error>
    where X : Serialize + Debug {
        let absolute_path = self.save_dir.join(Path::new(file));
        let mut f = File::create(absolute_path)?;
        println!("attempting to serialize & save {:?}", &x);
        f.write_all(
            & bincode::serialize(x, bincode::Infinite)
            .expect("couldn't serialize for saving.rs!")
        )?;
        Ok(())
    }

    pub fn load_me<X>(&self, file : &str) -> Result<X, io::Error>
    where X : DeserializeOwned {
        let absolute_path = self.save_dir.join(Path::new(file));
        let mut f = File::open(absolute_path)?;
        let mut buffer = vec![];
        f.read_to_end(&mut buffer)?;
        let z = bincode::deserialize(&buffer);
        if let Ok(x) = z {
            Ok(x)
        } else {
            Err(Error::new(ErrorKind::Other, "oh no!"))
        }
    }
}
