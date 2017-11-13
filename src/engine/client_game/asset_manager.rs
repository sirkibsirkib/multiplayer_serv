use std::collections::HashMap;
use ::identity::AssetID;
use super::piston_window::{G2dTexture,Texture,TextureSettings,Flip};
use super::piston_window::ImageSize;
use ::gfx_device_gl::Factory;

pub struct AssetManager {
    assets : HashMap<AssetID, G2dTexture>,
    factory : Factory,
}

impl AssetManager {
    fn aid_to_path(aid : AssetID) -> String {
        format!("./assets/asset_{}.png", aid)
    }

    pub fn new(factory : &Factory) -> AssetManager {
        AssetManager {
            assets : HashMap::new(),
            factory : factory.clone(),
        }
    }

    pub fn get_texture_for(&mut self, aid : AssetID) -> &G2dTexture {
        if ! self.assets.contains_key(&aid) {
            //load it first if you must
            let aid_path : &str = & Self::aid_to_path(aid);
            let texture : G2dTexture = Texture::from_path(
                &mut self.factory,
                aid_path,
                Flip::None,
                &TextureSettings::new()
            ).expect(& format!("Couldn't find file {:?} for aid {:?}", aid_path, aid));
            self.assets.insert(aid, texture);
        }
        self.assets.get(&aid).expect("zomg_wtf")
    }
}
