use std::collections::HashMap;
use ::identity::*;
use super::piston_window::{G2dTexture,Texture,TextureSettings,Flip};
use super::piston_window::ImageSize;
use ::gfx_device_gl::Factory;
use std::path::{Path,PathBuf};
use ::saving::SaverLoader;

pub struct HardcodedAssets {
    pub planet_mask: G2dTexture,
}

impl HardcodedAssets {
    pub fn new(factory : &mut Factory) -> Self {
        HardcodedAssets {
            planet_mask: try_load_asset(factory, Path::new("./assets/planet_mask.png"))
                .expect("Failed to load vital asset: Planet_Mask.png"),
        }
    }
}

pub struct AssetManager {
    assets : HashMap<AssetID, G2dTexture>,
    maps : HashMap<WorldID, G2dTexture>,
    saver_loader : SaverLoader,
    factory : Factory,
}

fn try_load_asset(factory: &mut Factory, path: &Path) -> Result<G2dTexture,()> {
    let x = Texture::from_path(
        factory,
        path,
        Flip::None,
        &TextureSettings::new()
    );
    if let Ok(q) = x { Ok(q) }
    else { Err(()) }
}

impl AssetManager {

    pub fn get_tex_size(&mut self, aid : AssetID) -> (u32,u32) {
        self.load_tex_if_missing(aid);
        let t = self.assets.get(&aid).unwrap();
        t.get_size()
    }

    pub fn get_tex_width(&mut self, aid : AssetID) -> u32 {
        self.get_tex_size(aid).0
    }

    pub fn aid_to_path(aid : AssetID) -> String {
        format!("./assets/asset_{}.png", aid)
    }
    pub fn wid_to_path(&self, wid : WorldID) -> PathBuf {
        self.saver_loader.relative_path(& format!("./temp_assets/worldmap_{}.png", wid))
    }

    pub fn new(factory : &Factory, saver_loader : SaverLoader) -> AssetManager {
        AssetManager {
            assets : HashMap::new(),
            maps : HashMap::new(),
            factory : factory.clone(),
            saver_loader: saver_loader,
        }
    }

    fn load_tex_if_missing(&mut self, aid : AssetID) {
        if ! self.assets.contains_key(&aid) {
            self.saver_loader.ensure_folder_exists("temp_assets/");
            let aid_path = & Self::aid_to_path(aid);
            let texture = try_load_asset(&mut self.factory, Path::new(aid_path))
            .expect(& format!("Couldn't find file {:?} for aid {:?}", aid_path, aid));
            self.assets.insert(aid, texture);
        }
    }

    pub fn has_map_for(&self, wid: WorldID) -> bool {
        self.maps.contains_key(&wid)
    }

    pub fn borrow_saver_loader(&self) -> &SaverLoader {
        &self.saver_loader
    }

    pub fn get_map_for(&mut self, wid : WorldID) -> Result<&G2dTexture,()> {
        if self.maps.contains_key(&wid) {
            return Ok(self.maps.get(&wid).expect("you lied to me"));
        }
        let path = &self.wid_to_path(wid);
        if let Ok(loaded) = try_load_asset(&mut self.factory, path) {
            self.maps.insert(wid, loaded);
            Ok(self.maps.get(&wid).expect("wtfmang"))
        } else {
            Err(())
        }
    }

    pub fn get_texture_for(&mut self, aid : AssetID) -> &G2dTexture {
        self.load_tex_if_missing(aid);
        self.assets.get(&aid).expect("zomg_wtf")
    }
}
