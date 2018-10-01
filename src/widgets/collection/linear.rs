use super::*;

pub enum ContentAlign {
    Leading,
    Stretching,
    Trailing,
}

pub struct TopToBottomLayout{
    align: ContentAlign,
}

pub struct LeftToRightLayout{
    align: ContentAlign,
}

pub struct BottomToTopLayout{
    align: ContentAlign,
}

pub struct RightToLeftLayout{
    align: ContentAlign,
}

impl TopToBottomLayout {
    pub fn new(align: ContentAlign) -> Self { Self { align } }
}

impl LeftToRightLayout {
    pub fn new(align: ContentAlign) -> Self { Self { align } }
}

impl BottomToTopLayout {
    pub fn new(align: ContentAlign) -> Self { Self { align } }
}

impl RightToLeftLayout {
    pub fn new(align: ContentAlign) -> Self { Self { align } }
}

impl Layouter for TopToBottomLayout {
    fn header(&self, _: usize, item: &Layout, parent: &Layout) -> Vec<cassowary::Constraint> {
        vec![
            item.top |EQ(REQUIRED)| parent.top,
            if self.align.anchor_leading() {
                item.left | EQ(REQUIRED) | parent.left
            } else {
                item.left | GE(REQUIRED) | parent.left
            },
            if self.align.anchor_trailing() {
                item.right | EQ(REQUIRED) | parent.right
            } else {
                item.right | LE(REQUIRED) | parent.right
            },
        ]
    }

    fn item(&self, _: usize, item: &Layout, parent: &Layout, previous: &Layout) -> Vec<cassowary::Constraint> {
        vec![
            item.top |EQ(REQUIRED)| previous.bottom,
            if self.align.anchor_leading() {
                item.left | EQ(REQUIRED) | parent.left
            } else {
                item.left | GE(REQUIRED) | parent.left
            },
            if self.align.anchor_trailing() {
                item.right | EQ(REQUIRED) | parent.right
            } else {
                item.right | LE(REQUIRED) | parent.right
            },
        ]
    }

    fn footer(&self, _: usize, item: &Layout, parent: &Layout) -> Vec<cassowary::Constraint> {
        vec![
            item.bottom |EQ(REQUIRED)| parent.bottom
        ]
    }
}

impl Layouter for LeftToRightLayout {
    fn header(&self, _: usize, item: &Layout, parent: &Layout) -> Vec<cassowary::Constraint> {
        vec![
            item.left |EQ(REQUIRED)| parent.left,
            if self.align.anchor_leading() {
                item.top | EQ(REQUIRED) | parent.top
            } else {
                item.top | GE(REQUIRED) | parent.top
            },
            if self.align.anchor_trailing() {
                item.bottom | EQ(REQUIRED) | parent.bottom
            } else {
                item.bottom | LE(REQUIRED) | parent.bottom
            },
        ]
    }

    fn item(&self, _: usize, item: &Layout, parent: &Layout, previous: &Layout) -> Vec<cassowary::Constraint> {
        vec![
            item.left |EQ(REQUIRED)| previous.right,
            if self.align.anchor_leading() {
                item.top | EQ(REQUIRED) | parent.top
            } else {
                item.top | GE(REQUIRED) | parent.top
            },
            if self.align.anchor_trailing() {
                item.bottom | EQ(REQUIRED) | parent.bottom
            } else {
                item.bottom | LE(REQUIRED) | parent.bottom
            },
        ]
    }

    fn footer(&self, _: usize, item: &Layout, parent: &Layout) -> Vec<cassowary::Constraint> {
        vec![
            item.right |EQ(REQUIRED)| parent.right
        ]
    }
}

impl Layouter for BottomToTopLayout {
    fn header(&self, _: usize, item: &Layout, parent: &Layout) -> Vec<cassowary::Constraint> {
        vec![
            item.bottom |EQ(REQUIRED)| parent.bottom,
            if self.align.anchor_leading() {
                item.left | EQ(REQUIRED) | parent.left
            } else {
                item.left | GE(REQUIRED) | parent.left
            },
            if self.align.anchor_trailing() {
                item.right | EQ(REQUIRED) | parent.right
            } else {
                item.right | LE(REQUIRED) | parent.right
            },
        ]
    }

    fn item(&self, _: usize, item: &Layout, parent: &Layout, previous: &Layout) -> Vec<cassowary::Constraint> {
        vec![
            item.bottom |EQ(REQUIRED)| previous.top,
            if self.align.anchor_leading() {
                item.left | EQ(REQUIRED) | parent.left
            } else {
                item.left | GE(REQUIRED) | parent.left
            },
            if self.align.anchor_trailing() {
                item.right | EQ(REQUIRED) | parent.right
            } else {
                item.right | LE(REQUIRED) | parent.right
            },
        ]
    }

    fn footer(&self, _: usize, item: &Layout, parent: &Layout) -> Vec<cassowary::Constraint> {
        vec![
            item.top |EQ(REQUIRED)| parent.top
        ]
    }
}

impl Layouter for RightToLeftLayout {
    fn header(&self, _: usize, item: &Layout, parent: &Layout) -> Vec<cassowary::Constraint> {
        vec![
            item.right |EQ(REQUIRED)| parent.right,
            if self.align.anchor_leading() {
                item.top | EQ(REQUIRED) | parent.top
            } else {
                item.top | GE(REQUIRED) | parent.top
            },
            if self.align.anchor_trailing() {
                item.bottom | EQ(REQUIRED) | parent.bottom
            } else {
                item.bottom | LE(REQUIRED) | parent.bottom
            },
        ]
    }

    fn item(&self, _: usize, item: &Layout, parent: &Layout, previous: &Layout) -> Vec<cassowary::Constraint> {
        vec![
            item.right |EQ(REQUIRED)| previous.left,
            if self.align.anchor_leading() {
                item.top | EQ(REQUIRED) | parent.top
            } else {
                item.top | GE(REQUIRED) | parent.top
            },
            if self.align.anchor_trailing() {
                item.bottom | EQ(REQUIRED) | parent.bottom
            } else {
                item.bottom | LE(REQUIRED) | parent.bottom
            },
        ]
    }

    fn footer(&self, _: usize, item: &Layout, parent: &Layout) -> Vec<cassowary::Constraint> {
        vec![
            item.left |EQ(REQUIRED)| parent.left
        ]
    }
}

impl ContentAlign {
    fn anchor_leading(&self) -> bool {
        match self {
            &ContentAlign::Trailing => false,
            &_ => true,
        }
    }

    fn anchor_trailing(&self) -> bool {
        match self {
            &ContentAlign::Leading => false,
            &_ => true,
        }
    }
}
