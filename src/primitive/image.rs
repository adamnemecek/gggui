use super::*;

/// An image in an im9 atlas texture.
#[derive(Clone,Debug)]
pub struct Image {
    /// The texture atlas identifier that this image resides in.
    pub texture: usize,
    /// The texcoords within the atlas that the image spans.
    pub texcoords: Rect,
    /// The physical size in pixels of the image.
    pub size: Rect,
}
