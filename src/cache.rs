use std::mem;
use std::collections::HashMap;

use rusttype;
use rusttype::{vector, point};
use image;
use smallvec::SmallVec;

use qtree::*;
use primitive::*;
use render::*;
use loadable::*;

type GlyphCache = rusttype::gpu_cache::Cache<'static>;
pub type Font = rusttype::Font<'static>;
pub type FontId = usize;

enum CachedResource {
    Patch(Patch),
    Image(Image),
    Font(Font,FontId),
}

enum TextureSlot {
    Atlas(QTree<()>),
    Big,
}

#[allow(dead_code)]
pub struct Cache {
    size: usize,
    glyphs: GlyphCache,
    textures: Vec<TextureSlot>,
    resources: HashMap<String, CachedResource>,
    updates: Vec<Update>,
    font_id_counter: usize,
}

impl Cache {
    pub fn new(size: usize) -> Cache {
        const SCALE_TOLERANCE: f32 = 0.1;
        const POSITION_TOLERANCE: f32 = 0.1;

        let glyphs = GlyphCache::new(
            (size/2) as u32, 
            (size/2) as u32, 
            SCALE_TOLERANCE, 
            POSITION_TOLERANCE
        );

        let mut atlas = QTree::new(size);
        atlas.insert((), size/2).unwrap();

        let atlas_create = Update::Texture {
            id: 0,
            size: [size as u32, size as u32],
            data: Vec::new(),
            atlas: true,
        };

        Cache {
            size: size,
            glyphs: glyphs,
            textures: vec![TextureSlot::Atlas(atlas)],
            resources: HashMap::new(),
            updates: vec![atlas_create],
            font_id_counter: 0
        }
    }

    pub fn take_updates(&mut self) -> Vec<Update> {
        mem::replace(&mut self.updates, Vec::new())
    }

    pub fn get_patch<L: Loadable>(&mut self, load: L) -> Patch {
        let key = load.uid();

        match self.resources.get(&key) {
            Some(entry) => match entry {
                &CachedResource::Patch(ref patch) => return patch.clone(),
                &CachedResource::Image(_) => panic!("Resource is of type 'Image', not 'Patch'"),
                &CachedResource::Font(_,_) => panic!("Resource is of type 'Font', not 'Patch'"),
            },
            _ => (),
        }

        let value = self.load_patch(&load);

        self.resources.insert(key, CachedResource::Patch(value.clone()));
        value
    }

    pub fn get_image<L: Loadable>(&mut self, load: L) -> Image {
        let key = load.uid();

        match self.resources.get(&key) {
            Some(entry) => match entry {
                &CachedResource::Image(ref image) => return image.clone(),
                &CachedResource::Patch(_) => panic!("Resource is of type 'Patch', not 'Image'"),
                &CachedResource::Font(_, _) => panic!("Resource is of type 'Font', not 'Image'"),
            },
            _ => (),
        }

        let value = self.load_image(&load);

        self.resources.insert(key, CachedResource::Image(value.clone()));
        value
    }

    pub fn get_font<L: Loadable>(&mut self, load: L) -> (Font, FontId) {
        let key = load.uid();

        match self.resources.get(&key) {
            Some(entry) => match entry {
                &CachedResource::Font(ref font, font_id) => return (font.clone(), font_id),
                &CachedResource::Patch(_) => panic!("Resource is of type 'Patch', not 'Font'"),
                &CachedResource::Image(_) => panic!("Resource is of type 'Image', not 'Font'"),
            },
            _ => (),
        }

        let value = self.load_font(&load);

        let font_id = self.font_id_counter;
        self.font_id_counter += 1;

        self.resources.insert(key, CachedResource::Font(value.clone(), font_id as FontId));
        (value, font_id)
    }

    pub fn draw_text<F: FnMut(Rect,Rect)>(
        &mut self, 
        text: &Text,
        rect: Rect,
        mut place_glyph: F
    ) {
        let start = point(rect.left, rect.top);
        
        let mut placed_glyphs = Vec::with_capacity(text.text.len());
        text.layout(rect, |g, x, _, y| {
            placed_glyphs.push(g.positioned(start + vector(x, y)));
        });

        for g in placed_glyphs.iter() {
            self.glyphs.queue_glyph(text.font.1 as usize, g.clone());
        }

        let updates = &mut self.updates;
        self.glyphs.cache_queued(|rect, data| {

            let mut new_data = Vec::with_capacity(data.len() * 4);
            for x in data {
                new_data.push(255);
                new_data.push(255);
                new_data.push(255);
                new_data.push(*x);
            }

            let update = Update::TextureSubresource {
                id: 0,
                offset: [rect.min.x, rect.min.y],
                size: [rect.width(), rect.height()],
                data: new_data,
            };

            updates.push(update);
        }).unwrap();

        for g in placed_glyphs.iter() {
            self.glyphs.rect_for(text.font.1 as usize, g).unwrap().map(|(uv, pos)| {
                place_glyph(
                    Rect {
                        left: uv.min.x * 0.5,
                        top: uv.min.y * 0.5,
                        right: uv.max.x * 0.5,
                        bottom: uv.max.y * 0.5,
                    },
                    Rect {
                        left: pos.min.x as f32,
                        top: pos.min.y as f32,
                        right: pos.max.x as f32,
                        bottom: pos.max.y as f32,
                    }
                );
            });
        }
    }

    fn insert_image(&mut self, image: image::RgbaImage) -> (usize, Rect) {
        let tex_id = 0;
        let (area, atlas_size) = if let TextureSlot::Atlas(ref mut atlas) = self.textures[tex_id] {
            let image_size = image.width().max(image.height()) as usize;
            (atlas.insert((), image_size).ok(), atlas.size() as f32)
        } else {
            (None, 0.0)
        };

        if area.is_some() {
            let mut area = area.unwrap();

            area.right = area.left + image.width() as usize;
            area.bottom = area.top + image.height() as usize;

            let update = Update::TextureSubresource {
                id: tex_id,
                offset: [area.left as u32, area.top as u32],
                size: [image.width(), image.height()],
                data: image.to_vec(),
            };
            self.updates.push(update);

            (tex_id, Rect{
                left: area.left as f32 / atlas_size,
                top: area.top as f32 / atlas_size,
                right: area.right as f32 / atlas_size,
                bottom: area.bottom as f32 / atlas_size,
            })
        } else {
            let tex_id = self.textures.len();

            let update = Update::Texture {
                id: tex_id,
                size: [image.width(), image.height()],
                data: image.to_vec(),
                atlas: false,
            };

            self.updates.push(update);
            self.textures.push(TextureSlot::Big);

            (tex_id, Rect::from_wh(1.0, 1.0))
        }
    }

    fn load_image<L: Loadable>(&mut self, load: &L) -> Image {
        // load image data
        let image_data = image::load(load.open(), image::ImageFormat::PNG).unwrap().to_rgba();

        let size = Rect{ 
            left: 0.0, 
            top: 0.0, 
            right: image_data.width() as f32, 
            bottom: image_data.height() as f32
        };
        let (texture, texcoords) = self.insert_image(image_data);

        Image { 
            texture, 
            texcoords, 
            size 
        }
    }

    fn load_patch<L: Loadable>(&mut self, load: &L) -> Patch {
        // load image data
        let mut image_data = image::load(load.open(), image::ImageFormat::PNG).unwrap().to_rgba();

        // find 9 patch borders in image data
        let black = image::Rgba{ data: [0u8, 0u8, 0u8, 255u8]};

        let mut h_stretch = SmallVec::<[(f32, f32); 2]>::new();
        let mut h_content = (1.0, 0.0);
        let mut v_stretch = SmallVec::<[(f32, f32); 2]>::new();
        let mut v_content = (1.0, 0.0);
        let mut h_current_stretch = None;
        let mut v_current_stretch = None;

        // scan horizontal stretch and content bars
        for x in 1..image_data.width()-1 {
            let h_begin = (x-1) as f32 / (image_data.width()-2) as f32;
            let h_end = (x) as f32 / (image_data.width()-2) as f32;

            // check stretch pixel
            if image_data[(x, 0)] == black {
                h_current_stretch = 
                    Some(h_current_stretch.map_or_else(|| (h_begin, h_end),|(s, _)| (s, h_end)));
            } else {
                h_current_stretch.take().map(|s| h_stretch.push(s));
            }

            // check content pixel
            if image_data[(x, image_data.height()-1)] == black {
                h_content.0 = h_begin.min(h_content.0);
                h_content.1 = h_end.max(h_content.1);
            }
        }

        // scan vertical stretch and content bars
        for y in 1..image_data.height()-1 {
            let v_begin = (y-1) as f32 / (image_data.height()-2) as f32;
            let v_end = (y) as f32 / (image_data.height()-2) as f32;

            // check stretch pixel
            if image_data[(0, y)] == black {
                v_current_stretch = 
                    Some(v_current_stretch.map_or_else(|| (v_begin, v_end),|(s, _)| (s, v_end)));
            } else {
                v_current_stretch.take().map(|s| v_stretch.push(s));
            }

            // check content pixel
            if image_data[(image_data.width()-1, y)] == black {
                v_content.0 = v_begin.min(v_content.0);
                v_content.1 = v_end.max(v_content.1);
            }
        }

        h_current_stretch.take().map(|s| h_stretch.push(s));
        v_current_stretch.take().map(|s| v_stretch.push(s));

        // strip stretch and content bars from the image
        let patch_width = image_data.width() - 2;
        let patch_height = image_data.height() - 2;
        let image_data = image::imageops::crop(&mut image_data, 1, 1, patch_width, patch_height)
            .to_image();
        let size = Rect{
            left: 0.0,
            top: 0.0,
            right: image_data.width() as f32, 
            bottom: image_data.height() as f32
        };
        let (texture, texcoords) = self.insert_image(image_data);

        Patch {
            image: Image{ texture, texcoords, size },
            h_stretch,
            v_stretch,
            h_content,
            v_content,
        }
    }

    fn load_font<L: Loadable>(&mut self, load: &L) -> Font {
        let collection = rusttype::FontCollection::from_bytes(load.bytes());
        let font = collection.into_font().unwrap();

        font
    }
}