use hashbrown::HashMap;

use super::{Renderer, Texture, TextureFormat};

pub type TextureAsset = u64;

pub struct AssetLoader {
    last_id: u64,
    textures: HashMap<TextureAsset, Texture>,
}

impl AssetLoader {
    pub fn new() -> Self {
        Self {
            last_id: 0,
            textures: HashMap::new(),
        }
    }

    fn new_id(&mut self) -> u64 {
        self.last_id += 1;

        self.last_id
    }

    pub fn load_texture(&mut self, renderer: &Renderer, width: u32, height: u32, texels: &[u8], format: TextureFormat) -> TextureAsset {
        let id = self.new_id();
        let texture = Texture::with_texels(renderer, width, height, texels, format);

        self.textures.insert(id, texture);

        id
    }

    pub fn texture(&self, id: TextureAsset) -> Option<&Texture> {
        self.textures.get(&id)
    }
}

impl Default for AssetLoader {
    fn default() -> Self {
        Self::new()
    }
}
