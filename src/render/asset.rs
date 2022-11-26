use alloc::vec::Vec;

use hashbrown::HashMap;

use super::{Renderer, Texture, TextureFormat};

pub type TextureAsset = u64;

struct TextureData {
    width: u32,
    height: u32,
    texels: Vec<u8>,
    format: TextureFormat,
}

pub struct AssetLoader {
    last_id: u64,
    textures: HashMap<TextureAsset, TextureData>,
    loaded_textures: HashMap<TextureAsset, Texture>,
}

impl AssetLoader {
    pub fn new() -> Self {
        Self {
            last_id: 0,
            textures: HashMap::new(),
            loaded_textures: HashMap::new(),
        }
    }

    fn new_id(&mut self) -> u64 {
        self.last_id += 1;

        self.last_id
    }

    pub fn load_texture(&mut self, width: u32, height: u32, texels: &[u8], format: TextureFormat) -> TextureAsset {
        let id = self.new_id();

        self.textures.insert(
            id,
            TextureData {
                width,
                height,
                texels: texels.to_vec(),
                format,
            },
        );

        id
    }

    pub fn texture(&mut self, renderer: &Renderer, id: TextureAsset) -> Option<&Texture> {
        if !(self.loaded_textures.contains_key(&id) || self.textures.contains_key(&id)) {
            return None;
        }

        let entry = self.loaded_textures.entry(id);

        Some(entry.or_insert_with(|| {
            let x = self.textures.get(&id).unwrap();

            Texture::with_texels(renderer, x.width, x.height, &x.texels, x.format)
        }))
    }
}

impl Default for AssetLoader {
    fn default() -> Self {
        Self::new()
    }
}
