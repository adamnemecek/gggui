use super::*;

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