use std::collections::HashMap;
use ::identity::*;
use ::engine::game_state::worlds::*;
use ::saving::SaverLoader;
use super::asset_manager::AssetManager;

#[derive(Debug)]
pub struct CacheManager {
    world_primitives : HashMap<WorldID, WorldPrimitive>,
    worlds : HashMap<WorldID, World>,
}

impl CacheManager {
    pub fn new(sl: SaverLoader) -> Self {
        CacheManager {
            world_primitives: HashMap::new(),
            worlds: HashMap::new(),
        }
    }

    pub fn ensure_map_file_exists_for(&mut self, wid: WorldID, am: &AssetManager) -> Result<(),()> {
        if am.borrow_saver_loader().file_folder_exists((&am.wid_to_path(wid).as_path()).to_str().unwrap()) {
            println!("File already exists, brah");
            return Ok(())
        }
        if ! self.worlds.contains_key(&wid) {
            if self.world_primitives.contains_key(&wid){
                let w = World::new(self.world_primitives.get(&wid).expect("hyeh").clone());
                self.worlds.insert(wid, w);
            }
        }
        if let Some(q) = self.worlds.get(&wid){
            q.to_png(&am.wid_to_path(wid), 300).is_ok();
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn world_is_cached(&self, wid: WorldID) -> bool {
        self.worlds.contains_key(&wid)
    }

    pub fn get_world(&self, wid: WorldID) -> Option<&World> {
        self.worlds.get(&wid)
    }

    pub fn cache_world_primitive(&mut self, wid: WorldID, wp: WorldPrimitive) {
        if ! self.world_primitives.contains_key(&wid){
            self.world_primitives.insert(wid, wp);
        }
    }
}
