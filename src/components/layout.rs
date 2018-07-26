use super::*;

#[derive(Clone, PartialEq)]
pub enum Constraint {
    Fixed,
    Grow,
    Fill,
}

#[derive(Clone)]
pub enum Gravity {
    Begin,
    Middle,
    End,
}

#[derive(Clone)]
pub struct Layout {
    pub left: cassowary::Variable,
    pub right: cassowary::Variable,
    pub top: cassowary::Variable,
    pub bottom: cassowary::Variable,
    pub center_x: cassowary::Variable,
    pub center_y: cassowary::Variable,
    pub width: cassowary::Variable,
    pub height: cassowary::Variable,
    constraints: Vec<cassowary::Constraint>,
    current: Option<Rect>,
}

use cassowary::strength::{WEAK, STRONG, REQUIRED};
use cassowary::WeightedRelation::*;

impl Layout {
    pub fn new() -> Self {
        let left     = cassowary::Variable::new();
        let right    = cassowary::Variable::new();
        let top      = cassowary::Variable::new();
        let bottom   = cassowary::Variable::new();
        let center_x = cassowary::Variable::new();
        let center_y = cassowary::Variable::new();
        let width    = cassowary::Variable::new();
        let height   = cassowary::Variable::new();

        Self {
            constraints: vec![
                left + width |EQ(REQUIRED)| right,
                top + height |EQ(REQUIRED)| bottom,
                center_x |EQ(REQUIRED)| (left+right)*0.5,
                center_y |EQ(REQUIRED)| (top+bottom)*0.5,
            ],
            current: None,

            left, right,
            top, bottom,
            center_x, 
            center_y,
            width, height
        }
    }

    pub fn with_intrinsic_size_constraints(mut self, width: f32, height: f32, hugging: f64) -> Self {
        // compression resistance
        self.constraints.push(self.width |GE(STRONG)| width);
        self.constraints.push(self.height |GE(STRONG)| height);
        // content hugging
        self.constraints.push(self.width |LE(WEAK+hugging)| width);
        self.constraints.push(self.height |LE(WEAK+hugging)| height);
        self
    }

    pub fn with_constraint(mut self, constraint: cassowary::Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    pub fn with_constraints(mut self, mut constraints: Vec<cassowary::Constraint>) -> Self {
        self.constraints.append(&mut constraints);
        self
    }

    pub fn current<'a>(&'a self) -> Option<&'a Rect> {
        self.current.as_ref()
    }
}