use smallvec::SmallVec;
use rusttype;
use super::Font;

#[derive(Clone,Copy,Debug)]
pub struct Rect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32
}

#[derive(Clone,Copy,Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn white() -> Color { Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 } }
    pub fn black() -> Color { Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 } }
    pub fn red() -> Color { Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 } }
    pub fn green() -> Color { Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 } }
    pub fn blue() -> Color { Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 } }
    pub fn with_alpha(mut self, a: f32) -> Self {
        self.a = a;
        self
    }
}

#[derive(Clone)]
pub struct Text {
    pub text: String,
    pub font: Font,
    pub size: f32,
}

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

/// 9 patch data on top of an `Image`, which is used to create dynamically stretchable images.
#[derive(Clone,Debug)]
pub struct Patch {
    /// The `Image` this `Patch` operates on.
    pub image: Image,
    /// Horizontally stretchable regions in the 9 patch image.
    /// Every element is a pair of begin and end of the stretchable region.
    /// Defined in relative coordinates: 0.0 is the left side of the image,
    /// 1.0 is the right side of the image.
    pub h_stretch: SmallVec<[(f32,f32); 2]>,
    /// Vertically stretchable regions in the 9 patch image.
    /// Every element is a pair of begin and end of the stretchable region.
    /// Defined in relative coordinates: 0.0 is the top side of the image,
    /// 1.0 is the bottom side of the image.
    pub v_stretch: SmallVec<[(f32,f32); 2]>,
    /// Horizontal content area in the 9 patch image. Content can be placed
    /// in the region defined here.
    /// Defined in relative coordinates: 0.0 is the left side of the image,
    /// 1.0 is the right side of the image.
    pub h_content: (f32,f32),
    /// Vertical content area in the 9 patch image. Content can be placed
    ///  in the region defined here.
    /// Defined in relative coordinates: 0.0 is the top side of the image,
    /// 1.0 is the bottom side of the image.
    pub v_content: (f32,f32),
}

pub enum Primitive {
    PushClip(Rect),
    PopClip,
    DrawRect(Rect, Color),
    DrawText(Text, Rect, Color, bool),
    Draw9(Patch, Rect, Color),
    DrawImage(Image, Rect, Color),
}

impl Text {
    pub fn measure(&self, width: Option<f32>) -> Rect {
        let scale = rusttype::Scale{ x: self.size, y: self.size };
        let line = self.font.0.v_metrics(scale);

        let mut last = None;
        let mut x = 0.0;
        let mut y = line.ascent;
        let mut max_x = 0.0;

        for g in self.text.chars().map(|c| self.font.0.glyph(c).unwrap()) {
            let g = g.scaled(scale);
            let w = g.h_metrics().advance_width
                + last.map(|last| self.font.0.pair_kerning(scale, last, g.id())).unwrap_or(0.0);

            width.map(|width| {
                if x + w > width {
                    x = 0.0;
                    y += line.descent+line.line_gap+line.ascent;
                }
            });

            //let next = g.positioned(start + vector(x, y));
            last = Some(g.id());

            x += w;

            if x > max_x { 
                max_x = x; 
            }
        }

        Rect{ 
            left: 0.0, 
            top: 0.0, 
            right: width.unwrap_or(max_x).ceil(), 
            bottom: (y+line.descent).ceil() 
        }
    }

    pub fn measure_range(
        &self, 
        from: usize, 
        to: usize, 
        width: Option<f32>
    ) -> ((f32,f32), (f32,f32)) {
        let scale = rusttype::Scale{ x: self.size, y: self.size };
        let line = self.font.0.v_metrics(scale);

        let mut last = None;
        let mut x = 0.0;
        let mut y = line.ascent;

        let mut from_result = (0.0, 0.0);
        let mut to_result = (0.0, 0.0);

        for (i, g) in self.text.char_indices().map(|(i, c)| (i, self.font.0.glyph(c).unwrap())) {
            let g = g.scaled(scale);
            let w = g.h_metrics().advance_width
                + last.map(|last| self.font.0.pair_kerning(scale, last, g.id())).unwrap_or(0.0);

            width.map(|width| {
                if x + w > width {
                    x = 0.0;
                    y += line.descent+line.line_gap+line.ascent;
                }
            });

            //let next = g.positioned(start + vector(x, y));
            last = Some(g.id());

            if i == from {
                from_result = (x, y);
            }
            if i == to {
                to_result = (x, y);
                break;
            }

            x += w;

            if i+1 == from {
                from_result = (x, y);
            }
            if i+1 == to {
                to_result = (x, y);
                break;
            }
        }

        (from_result, to_result)
    }

    pub fn hitdetect(&self, cursor: (f32, f32), width: Option<f32>) -> usize {
        let scale = rusttype::Scale{ x: self.size, y: self.size };
        let line = self.font.0.v_metrics(scale);

        let mut last = None;
        let mut x = 0.0;
        let mut y = line.ascent;

        let mut nearest = (cursor.0, 0);

        for (i, g) in self.text.char_indices().map(|(i, c)| (i, self.font.0.glyph(c).unwrap())) {
            let g = g.scaled(scale);
            let w = g.h_metrics().advance_width
                + last.map(|last| self.font.0.pair_kerning(scale, last, g.id())).unwrap_or(0.0);

            width.map(|width| {
                if x + w > width {
                    x = 0.0;
                    y += line.descent+line.line_gap+line.ascent;
                }
            });

            //let next = g.positioned(start + vector(x, y));
            last = Some(g.id());

            if (x-cursor.0).abs() < nearest.0 { 
                nearest.0 = (x-cursor.0).abs();
                nearest.1 = i;
            }

            x += w;

            if (x-cursor.0).abs() < nearest.0 { 
                nearest.0 = (x-cursor.0).abs();
                nearest.1 = i+1;
            }
        }

        nearest.1
    }
}

impl Rect {
    pub fn to_device_coordinates(self, viewport: Rect) -> Rect {
        let center = ((viewport.left+viewport.right)*0.5, (viewport.top+viewport.bottom)*0.5);
        let size = ((viewport.right-viewport.left)*0.5, (viewport.top-viewport.bottom)*-0.5);
        Rect {
            left: (self.left - center.0) / size.0,
            top: (self.top - center.1) / size.1,
            right: (self.right - center.0) / size.0,
            bottom: (self.bottom - center.1) / size.1,
        }
    }

    pub fn from_wh(w: f32, h: f32) -> Rect {
        Rect {
            left: 0.0,
            right: w,
            top: 0.0,
            bottom: h
        }
    }

    pub fn intersect(&self, other: &Rect) -> Option<Rect> {
        let result = Rect {
            left: self.left.max(other.left),
            top: self.top.max(other.top),
            right: self.right.min(other.right),
            bottom: self.bottom.min(other.bottom),
        };
        if result.left < result.right && result.top < result.bottom {
            Some(result)
        } else {
            None
        }
    }

    pub fn pt(&self, x: f32, y: f32) -> [f32;2] {
        [self.left+(self.right-self.left)*x, self.top+(self.bottom-self.top)*y]
    }

    pub fn round(self) -> Rect {
        Rect {
            left: self.left.round(),
            top: self.top.round(),
            right: self.right.round(),
            bottom: self.bottom.round(),
        }
    }

    pub fn sub(&self, lerps: Rect) -> Rect {
        Rect {
            left: self.left + (self.right-self.left) * lerps.left,
            right: self.left + (self.right-self.left) * lerps.right,
            top: self.top + (self.bottom-self.top) * lerps.top,
            bottom: self.top + (self.bottom-self.top) * lerps.bottom,
        }
    }

    pub fn translate(&self, x: f32, y: f32) -> Rect {
        Rect {
            left: self.left + x,
            top: self.top + y,
            right: self.right + x,
            bottom: self.bottom + y,
        }
    }

    pub fn grow(&self, w: f32, h: f32) -> Rect {
        Rect {
            left: self.left,
            top: self.top,
            right: self.right + w,
            bottom: self.bottom + h,
        }
    }

    pub fn width(&self) -> f32 {
        self.right-self.left
    }

    pub fn height(&self) -> f32 {
        self.bottom-self.top
    }
}

impl Into<Rect> for [f32;4] {
    fn into(self) -> Rect {
        Rect {
            left: self[0],
            top: self[1],
            right: self[2],
            bottom: self[3],
        }
    }
}

impl Patch {
    pub fn measure_with_content(&self, measured_content: Rect) -> Rect {
        let patch_content = self.image.size.sub(Rect{ 
            left: self.h_content.0, 
            right: self.h_content.1,
            top: self.v_content.0,
            bottom: self.v_content.1,
        });

        let grow_x = (measured_content.width() - patch_content.width()).max(0.0);
        let grow_y = (measured_content.height() - patch_content.height()).max(0.0);

        let result = Rect {
            left: 0.0,
            top: 0.0,
            right: self.image.size.width() + grow_x,
            bottom: self.image.size.height() + grow_y,
        };

        result
    }
    pub fn content_rect(&self, span: Rect) -> Rect {
        let mut result = span;

        let blend = |(a,b),x| a+(b-a)*x;
        let unblend = |x,(a,b)| (x-a)/(b-a);

        self.iterate_sections(false, span.width(), |x, u| {
            if self.h_content.0 >= u.0 && self.h_content.0 < u.1 {
                result.left = span.left + blend(x, unblend(self.h_content.0, u));
            }
            if self.h_content.1 > u.0 && self.h_content.1 <= u.1 {
                result.right = span.left + blend(x, unblend(self.h_content.1, u));
            }
        });
        self.iterate_sections(true, span.height(), |y, v| {
            if self.v_content.0 >= v.0 && self.v_content.0 < v.1 {
                result.top = span.top + blend(y, unblend(self.v_content.0, v));
            }
            if self.v_content.1 > v.0 && self.v_content.1 <= v.1 {
                result.bottom = span.top + blend(y, unblend(self.v_content.1, v));
            }
        });

        result
    }

    pub fn iterate_sections<
        F: FnMut((f32, f32), (f32, f32))
    > (
        &self,
        vertical: bool, 
        length: f32,
        mut callback: F
    ) {
        let stretches = if vertical {
            &self.v_stretch
        } else {
            &self.h_stretch
        };

        let total = stretches.iter().fold(0.0, |t, &(a,b)| t+(b-a));

        let mut cursor = 0.0;
        let mut grow = 0.0;

        let base = if vertical {
            (0.0, self.image.size.height())
        } else {
            (0.0, self.image.size.width())
        };

        let sub = |x| base.0+(base.1-base.0)*x;

        let space = length - base.1;

        for s in stretches.iter() {
            if s.0 > 0.0 {
                callback((sub(cursor) + grow, sub(s.0) + grow), (cursor, s.0));
            }

            let stretch = (s.1-s.0)/total*space;

            callback((sub(s.0) + grow, sub(s.1) + grow + stretch), (s.0, s.1));
            cursor = s.1;
            grow += stretch;
        }
        if cursor < 1.0 {
            callback((sub(cursor) + grow, sub(1.0) + grow), (cursor, 1.0));
        }
    }
}
