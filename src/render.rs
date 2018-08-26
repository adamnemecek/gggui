use super::primitive::*;
use std::cell::RefCell;

/// A collection of data needed to render the `Ui` once.
pub struct DrawList {
    /// A list of texture updates that need to be uploaded before rendering.
    pub updates: RefCell<Vec<Update>>,
    /// The vertex buffer used for this frame.
    pub vertices: Vec<Vertex>,
    /// A list of draw commands that use the `vertices` buffer.
    pub commands: Vec<Command>,
}

/// An update of the available texture data. The backend is reponsible for uploading the provided
/// data to the GPU.
pub enum Update {
    /// A subresource of an existing texture is updated. This happens f.e. when new glyphs are 
    /// loaded or when a new 9 patch is used.
    TextureSubresource {
        /// The id of the texture that needs to be updated
        id: usize,
        /// Offset from the left top corner of the texture.
        offset: [u32; 2],
        /// Size of the rect described by `data`
        size: [u32; 2],
        /// The texel data of the updated rect. 4 elements per pixel.
        data: Vec<u8>,
    },
    /// A new texture is introduced. This happens when f.e. a background image was loaded, or when
    /// the `Ui` is used for the first time.
    Texture {
        /// The id for the new texture. This is the id that will later be used to identify which
        /// texture the backend has to use whenever applicable.
        id: usize,
        /// Size of the texture
        size: [u32; 2],
        /// The texel data of the texture. 4 elements per pixel
        data: Vec<u8>,
        /// Whether the texture will be used as atlas. `true` means the texture might be updated
        /// later with `Update::TextureSubresource`, while `false` means the texture is immutable.
        atlas: bool,
    }
}

/// The `Vertex` type passed to the vertex shader.
#[derive(Debug, Clone, Copy)]
pub struct Vertex { 
    /// The position of the vertex within device coordinates.
    /// [-1.0, -1.0] is the left top position of the display.
    pub pos:   [f32; 2],
    /// The coordinates of the texture used by this `Vertex`.
    /// [0.0, 0.0] is the left top position of the texture.
    pub uv:    [f32; 2],
    /// A color associated with the `Vertex`.
    /// The color is multiplied by the end result of the fragment shader.
    /// When `mode` is not 1, the default value is white ([1.0; 4])
    pub color: [f32; 4],
    /// The mode with which the `Vertex` will be drawn within the fragment shader.
    ///
    /// `0` for rendering text.
    /// `1` for rendering an image.
    /// `2` for rendering non-textured 2D geometry.
    ///
    /// If any other value is given, the fragment shader will not output any color.
    pub mode:  u32, 
}

/// A draw `Command` that is to be translated to a draw command specific to the backend
#[derive(Debug, Clone, Copy)]
pub enum Command {
    /// Do nothing. Appending a `Nop` to another command will flush the other command.
    Nop,
    /// Sets a new scissor rect, which is used to confine geometry to a certain area on screen.
    Clip{ scissor: Rect },
    /// Draw a list of vertices without an active texture
    Colored{ offset: usize, count: usize },
    /// Draw a list of vertices with the active texture denoted by it's index
    Textured{ texture: usize, offset: usize, count: usize },
}

impl Command {
    /// Append another `Command` to this `Command`. If the `Command`s can be chained together
    /// the `Command` is extended and `None` is returned, but if the `Command`s can not be chained
    /// `self` is replaced by the new `Command` and the old `Command` is returned.
    pub fn append(&mut self, command: Command) -> Option<Command> {
        match *self {
            Command::Nop => {
                *self = command;
                None
            },

            Command::Clip{ scissor } => match command {
                Command::Nop => None,
                Command::Clip{ scissor: new_scissor } => {
                    *self = Command::Clip{ scissor: new_scissor };
                    None
                },
                Command::Colored{ offset, count } => {
                    *self = Command::Colored{ offset, count };
                    Some(Command::Clip{ scissor })
                },
                Command::Textured{ texture, offset, count } => {
                    *self = Command::Textured{ texture, offset, count };
                    Some(Command::Clip{ scissor })
                },
            },

            Command::Colored{ offset, count } => match command {
                Command::Nop => {
                    *self = Command::Nop;
                    Some(Command::Colored{ offset, count })
                },
                Command::Clip{ scissor } => {
                    *self = Command::Clip { 
                        scissor: scissor 
                    };
                    Some(Command::Colored{ offset, count })
                },
                Command::Colored{ offset: new_offset , count: new_count } => {
                    if new_offset == offset+count {
                        *self = Command::Colored { 
                            offset: offset, 
                            count: count+new_count 
                        };
                        None
                    } else {
                        *self = Command::Colored { 
                            offset: new_offset, 
                            count: new_count 
                        };
                        Some(Command::Colored{ offset, count })
                    }
                },
                Command::Textured{ texture, offset: new_offset, count: new_count } => {
                    if new_offset == offset+count {
                        *self = Command::Textured { 
                            texture: texture, 
                            offset: offset, 
                            count: count+new_count 
                        };
                        None
                    } else {
                        *self = Command::Textured { 
                            texture: texture, 
                            offset: new_offset, 
                            count: new_count 
                        };
                        Some(Command::Colored{ offset, count })
                    }
                },
            },

            Command::Textured{ texture, offset, count } => match command {
                Command::Nop => {
                    *self = Command::Nop;
                    Some(Command::Textured{ texture, offset, count })
                },
                Command::Clip{ scissor } => {
                    *self = Command::Clip{ 
                        scissor: scissor 
                    };
                    Some(Command::Textured{ texture, offset, count })
                },
                Command::Colored{ offset: new_offset , count: new_count } => {
                    if new_offset == offset+count {
                        *self = Command::Textured { 
                            texture: texture, 
                            offset: offset, 
                            count: count+new_count 
                        };
                        None
                    } else {
                        *self = Command::Colored { 
                            offset: new_offset, 
                            count: new_count 
                        };
                        Some(Command::Textured{ texture, offset, count })
                    }
                },
                Command::Textured{ texture: new_texture, offset: new_offset, count: new_count } => {
                    if texture == new_texture && new_offset == offset+count {
                        *self = Command::Textured { 
                            texture: texture, 
                            offset: offset, 
                            count: count+new_count 
                        };
                        None
                    } else {
                        *self = Command::Textured { 
                            texture: new_texture, 
                            offset: new_offset, 
                            count: new_count 
                        };
                        Some(Command::Textured{ texture, offset, count })
                    }
                },
            },
        }
    }

    /// Return any draw command that is still being built by the `Command`.
    /// This function is the same as `append(Command::Nop)`.
    pub fn flush(&mut self) -> Option<Command> {
        self.append(Command::Nop)
    }
}