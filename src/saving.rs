use std::io;
use std::io::prelude::*;
use std::fs::File;

use serde::Serialize;
use serde::de::DeserializeOwned;
use bincode;
use std::path::{Path,PathBuf};
use std::io::{ErrorKind,Error};
use std::fs::create_dir;
use std::fmt::Debug;
use ::network::userbase::UserBase;
use utils::traits::{KnowsSaveSuffix,KnowsSavePrefix};

#[derive(Clone,Debug)]
pub struct SaverLoader {
    save_dir : Box<PathBuf>,
}

impl SaverLoader {
    pub fn new(save_dir : &str) -> SaverLoader {
        let p = Path::new(save_dir);
        let me = SaverLoader {
            save_dir : Box::new(p.to_path_buf())
        };
        me.ensure_folder_exists("./");
        me.ensure_folder_exists("locations/");
        me.ensure_folder_exists(UserBase::REGISTER_PATH);
        me
    }

    pub fn relative_path<'a>(&self, rel : &'a str) -> PathBuf {
        self.save_dir.clone().join(Path::new(rel))
    }

    pub fn file_folder_exists(&self, path : &str) -> bool {
        self.relative_path(path).exists()
    }

    pub fn ensure_folder_exists(&self, path : &str) {
        let p = self.relative_path(path);

        if ! p.exists() {
            println!("CREATING NEW DIR for {:?}", &p);
            create_dir(p).expect("Couldn't create new save dir");
        }
    }

    fn save_me<X>(&self, x : &X, file : &str) -> Result<(), io::Error>
    where X : Serialize + Debug {
        let absolute_path = self.save_dir.join(Path::new(file));
        let mut f = File::create(absolute_path)?;
        f.write_all(
            & bincode::serialize(x, bincode::Infinite)
            .expect("couldn't serialize for saving.rs!")
        )?;
        Ok(())
    }

    pub fn save<X,K>(&self, x: &X, key: K) -> Result<(),io::Error>
    where X: Serialize + Debug + KnowsSavePrefix,
          K: KnowsSaveSuffix {
        self.save_me(x, &format!("{}{}", X::prefix_from(), key.suffix_from()))
    }

    fn load_me<X>(&self, file : &str) -> Result<X, io::Error>
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

    pub fn load<X,K>(&self, key: K) -> Result<X,io::Error>
    where X: DeserializeOwned + KnowsSavePrefix,
          K: KnowsSaveSuffix {
        self.load_me(&format!("{}{}", X::prefix_from(), key.suffix_from()))
    }
}
