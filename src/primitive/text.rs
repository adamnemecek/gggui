use super::*;
use std::f32;

#[derive(Clone,Copy,Debug)]
pub enum TextWrap {
    NoWrap,
    Wrap,
    WordWrap,
}

#[derive(Clone)]
pub struct Text {
    pub text: String,
    pub font: Font,
    pub size: f32,
    pub wrap: TextWrap,
}

pub struct CharPositionIter<'a, 'b: 'a> {
    font: &'b rusttype::Font<'static>,
    scale: rusttype::Scale,
    last: Option<rusttype::GlyphId>,
    x: f32,
    base: Chars<'a>,
}

impl<'a, 'b> Iterator for CharPositionIter<'a, 'b> {
    type Item = (char, rusttype::ScaledGlyph<'static>, f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        self.base.next().and_then(|c| {
            let g = self.font.glyph(c).unwrap();
            let g = g.scaled(self.scale);
            let w = g.h_metrics().advance_width 
                + self.last.map(|last| {
                    self.font.pair_kerning(self.scale, last, g.id())
                }).unwrap_or(0.0);

            self.last = Some(g.id());

            let elem = (c, g, self.x, self.x+w);
            self.x += w;
            Some(elem)
        })
    }
}

impl Text {
    pub fn char_positions<'a,'b>(&'b self) -> CharPositionIter<'a, 'b> {
        let scale = rusttype::Scale{ x: self.size, y: self.size };
        CharPositionIter {
            font: &self.font.0,
            scale: scale,
            last: None,
            x: 0.0,
            base: self.text.chars()
        }
    }

    pub fn layout<
        F: FnMut(rusttype::ScaledGlyph<'static>, f32, f32, f32)
    > (
        &self, 
        rect: Rect, 
        mut f: F
    ) {
        let line = self.font.0.v_metrics(rusttype::Scale{ x: self.size, y: self.size });

        let width = rect.width();

        match self.wrap {
            TextWrap::NoWrap => {
                for (_, g, a, b) in self.char_positions() {
                    f(g, a, b, line.ascent);
                }
            },

            TextWrap::Wrap => {
                let mut x = 0.0;
                let mut y = line.ascent;

                for (_, g, a, b) in self.char_positions() {
                    if b - x > width {
                        x = a;
                        y += -line.descent + line.line_gap + line.ascent;
                    }

                    f(g, a - x, b - x, y);
                }
            },

            TextWrap::WordWrap => {
                let mut x = 0.0;
                let mut y = line.ascent;
                let mut word_iter = self.char_positions().peekable();
                let mut word_started = false;

                for (c, g, a, b) in self.char_positions() {
                    if !c.is_alphanumeric() {
                        word_iter.next();
                        word_started = false;
                    } else {
                        if !word_started {
                            word_started = true;
                            let mut word_width = 0.0;
                            while let Some(&(c,_,_,w)) = word_iter.peek() {
                                if !c.is_alphanumeric() {
                                    break;
                                } else {
                                    word_width = w - x;
                                    word_iter.next();
                                }
                            }
                            
                            if word_width > width {
                                x = a;
                                y += -line.descent + line.line_gap + line.ascent;
                            }
                        }
                    }

                    if b - x > width {
                        x = a;
                        y += -line.descent + line.line_gap + line.ascent;
                    }

                    f(g, a - x, b - x, y);
                }
            },
        }
    }

    pub fn measure(&self, rect: Option<Rect>) -> Rect {
        let line = self.font.0.v_metrics(rusttype::Scale{ x: self.size, y: self.size });

        rect.map_or_else(
            || {
                let mut w = 0.0;
                self.layout(Rect::from_wh(f32::INFINITY, 0.0), |_,_,new_w,_| w = new_w);

                Rect::from_wh(w.ceil(), (line.ascent - line.descent).ceil())
            },
            |r| {
                let mut w = 0.0;
                let mut h = line.ascent;
                match self.wrap {
                    TextWrap::NoWrap => {
                        self.layout(r, |_,_,new_w,_| w = new_w)
                    },
                    TextWrap::Wrap | TextWrap::WordWrap => {
                        w = rect.map_or(0.0, |r| r.width());
                        self.layout(r, |_,_,_,new_h| h = new_h);
                    },
                }

                Rect::from_xywh(r.left, r.top, w.ceil(), (h - line.descent).ceil())
            })
    }

    pub fn measure_range(&self, from: usize, to: usize, rect: Rect) -> ((f32,f32), (f32,f32)) {
        let mut from_result = (0.0, 0.0);
        let mut to_result = (0.0, 0.0);

        let mut index = 0;
        self.layout(rect, |_, begin, end, y| {
            if index == from { from_result = (begin, y) }
            if index == to { to_result = (begin, y) }
            if index+1 == from { from_result = (end, y) }
            if index+1 == to { to_result = (end, y) }
            index += 1;
        });

        (from_result, to_result)
    }

    pub fn hitdetect(&self, cursor: (f32, f32), rect: Rect) -> usize {
        let dist = |(x,y)| x*x+y*y;

        let mut nearest = (dist(cursor), 0);
        let mut index = 0;

        self.layout(rect, |_,begin,end,y| {
             if dist((begin-cursor.0, y-cursor.1)) < nearest.0 { 
                nearest.0 = dist((begin-cursor.0, y-cursor.1));
                nearest.1 = index;
            }
            if dist((end-cursor.0, y-cursor.1)) < nearest.0 { 
                nearest.0 = dist((end-cursor.0, y-cursor.1));
                nearest.1 = index+1;
            }

            index += 1;
        });

        nearest.1
    }
}