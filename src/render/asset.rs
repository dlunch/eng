use core::cell::{RefCell, RefMut};

use hashbrown::HashMap;

use super::{Renderer, Texture, TextureFormat};

pub type TextureAsset = u64;

struct TextureData {
    width: u32,
    height: u32,
    texels: Box<[u8]>,
    format: TextureFormat,
}

pub struct AssetLoader {
    last_id: u64,
    textures: HashMap<TextureAsset, TextureData>,
    loaded_textures: RefCell<HashMap<TextureAsset, Texture>>,
}

impl AssetLoader {
    pub fn new() -> Self {
        Self {
            last_id: 0,
            textures: HashMap::new(),
            loaded_textures: RefCell::new(HashMap::new()),
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
                texels: texels.into(),
                format,
            },
        );

        id
    }

    // should this function return Ref<Texture>?
    pub fn texture(&self, renderer: &Renderer, id: TextureAsset) -> Option<RefMut<Texture>> {
        let loaded_textures = self.loaded_textures.borrow_mut();

        if !(loaded_textures.contains_key(&id) || self.textures.contains_key(&id)) {
            return None;
        }

        Some(RefMut::map(loaded_textures, |x| {
            let entry = x.entry(id);

            entry.or_insert_with(|| {
                let x = self.textures.get(&id).unwrap();

                Texture::with_texels(renderer, x.width, x.height, &x.texels, x.format)
            })
        }))
    }
}

impl Default for AssetLoader {
    fn default() -> Self {
        Self::new()
    }
}
