use std::io;
use std::io::prelude::*;
use std::fs::File;

use serde::{Serialize,Deserialize};
use serde::de::DeserializeOwned;
use serde_json;
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
            serde_json::to_string(x)
            .expect("couldn't serialize for saving.rs!")
            .as_bytes()
        )?;
        Ok(())
    }

    pub fn load_me<X>(&self, file : &str) -> Result<X, io::Error>
    where X : DeserializeOwned {
        let absolute_path = self.save_dir.join(Path::new(file));
        let mut f = File::open(absolute_path)?;
        let mut buffer = String::new();
        f.read_to_string(&mut buffer)?;
        let z = serde_json::from_str(&buffer);
        if let Ok(x) = z {
            Ok(x)
        } else {
            Err(Error::new(ErrorKind::Other, "oh no!"))
        }
    }
}


// pub trait SaveLoad<'a> : Serialize + Deserialize<'a> {
//     fn save_to(&self, dir : &Path, filename : &str) -> Result<(), io::Error> ;
//
//     fn load_from(dir : &Path, filename : &str) -> Result<Self, io::Error> ;
//
//     fn resolve_path(dir : &Path, filename : &str) -> PathBuf ;
// }
//
// impl<'a> SaveLoad<'a> for super::network::UserBase {
//     fn save_to(&self, dir : &Path, filename : &str) -> Result<(), io::Error> {
//         let mut f = File::create(Self::resolve_path(dir, filename))?;
//         f.write_all(
//             serde_json::to_string(self)
//             .expect("couldn't serialize for saving.rs!")
//             .as_bytes()
//         )?;
//         Ok(())
//     }
//
//     fn load_from(dir : &Path, filename : &str) -> Result<Self, io::Error> {
//         let mut f = File::open(Self::resolve_path(dir, filename))?;
//         let mut buffer = String::new();
//         f.read_to_string(&mut buffer)?;
//         let z = serde_json::from_str(&buffer);
//         if let Ok(x) = z {
//             Ok(x)
//         } else {
//             Err(Error::new(ErrorKind::Other, "oh no!"))
//         }
//     }
//
//     #[inline]
//     fn resolve_path(dir : &Path, filename : &str) -> PathBuf {
//         dir.join(Path::new(filename))
//     }
// }
