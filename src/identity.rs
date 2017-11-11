use serde::{Serialize,Deserialize};
use std::fmt::{Debug,Formatter};
use std::io::Write;
use std::collections::HashSet;

use std;
pub type EntityID = u64;
pub type LocationID = u32;
pub type ClientID = u16;
pub type AssetID = u16;
pub type SuperSeed = u64;

//////////////////////////////////////////////////////////////////////////////////

#[derive(Copy,Clone,Serialize,Deserialize)]
pub struct ClientIDSet {
    bits : u32,
}

impl ClientIDSet {
    const CAPACITY : ClientID = 32;

    #[inline]
    pub fn new() -> ClientIDSet {
        ClientIDSet {
            bits : 0,
        }
    }

    #[inline]
    pub fn get(&self, element : ClientID) -> bool {
        ! self.is_empty() //checking this is lightning fast
        && ((1 << element) & self.bits) > 0
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }

    pub fn set(&mut self, element : ClientID, pos : bool) {
        if element >= Self::CAPACITY {
            panic!("ClientIDSet CANT HANDLE THAT!");
        }
        if pos {
            //put it up
            self.bits = (1 << element) | self.bits;
        } else {
            self.bits = (std::u32::MAX - (1 << element)) & self.bits;
        }
    }

    pub fn iter_set_pos(self) -> ClientIDSetIntoIterator {
        ClientIDSetIntoIterator { pos_mode : true, bit_set: self, index: 0 }
    }

    pub fn iter_set_neg(self) -> ClientIDSetIntoIterator {
        ClientIDSetIntoIterator { pos_mode : false, bit_set: self, index: 0 }
    }
}

// impl IntoIterator for ClientIDSet {
//     type Item = ClientID;
//     type IntoIter = ClientIDSetIntoIterator;
//
//     fn into_iter(self) -> ClientIDSetIntoIterator {
//         ClientIDSetIntoIterator { bit_set: self, index: 0 }
//     }
// }

pub struct ClientIDSetIntoIterator {
    pos_mode : bool,
    bit_set : ClientIDSet,
    index : ClientID,
}

impl Iterator for ClientIDSetIntoIterator {
    type Item = ClientID;
    fn next(&mut self) -> Option<ClientID> {
        while self.index < ClientIDSet::CAPACITY {
            self.index += 1;
            if self.bit_set.get(self.index-1) == self.pos_mode {
                return Some(self.index-1)
            }
        }
        None
    }
}



impl Debug for ClientIDSet {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        let x : HashSet<ClientID> = self.iter_set_pos().collect();
        let _ = write!(f, "{:?}", &x);
        Ok(())
    }
}
