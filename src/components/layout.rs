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
    pub margin_left: cassowary::Variable,
    pub margin_right: cassowary::Variable,
    pub margin_top: cassowary::Variable,
    pub margin_bottom: cassowary::Variable,
    constraints: Vec<cassowary::Constraint>,
    pub current: Option<Rect>,
}

use cassowary::strength::{WEAK, STRONG, REQUIRED};
use cassowary::WeightedRelation::*;

impl Layout {
    pub fn new() -> Self {
        let left          = cassowary::Variable::new();
        let right         = cassowary::Variable::new();
        let top           = cassowary::Variable::new();
        let bottom        = cassowary::Variable::new();
        let center_x      = cassowary::Variable::new();
        let center_y      = cassowary::Variable::new();
        let width         = cassowary::Variable::new();
        let height        = cassowary::Variable::new();
        let margin_left   = cassowary::Variable::new();
        let margin_right  = cassowary::Variable::new();
        let margin_top    = cassowary::Variable::new();
        let margin_bottom = cassowary::Variable::new();

        Self {
            constraints: vec![
                left + width |EQ(REQUIRED)| right,
                top + height |EQ(REQUIRED)| bottom,
                center_x |EQ(REQUIRED)| (left+right)*0.5,
                center_y |EQ(REQUIRED)| (top+bottom)*0.5,

                margin_left |EQ(REQUIRED)| left,
                margin_right |EQ(REQUIRED)| right,
                margin_top |EQ(REQUIRED)| top,
                margin_bottom |EQ(REQUIRED)| bottom,
            ],
            current: None,

            left, right,
            top, bottom,
            center_x, center_y,
            width, height,
            margin_left, margin_right,
            margin_top, margin_bottom,
        }
    }

    pub fn as_editable(self, solver: &mut cassowary::Solver) -> Self {
        solver.add_edit_variable(self.left, STRONG);
        solver.add_edit_variable(self.top, STRONG);
        solver.add_edit_variable(self.right, STRONG);
        solver.add_edit_variable(self.bottom, STRONG);
        self
    }

    pub fn with_margins(mut self, margin: Rect) -> Self {
        self.constraints[4] = self.margin_left |EQ(REQUIRED)| self.left + margin.left as f64;
        self.constraints[5] = self.margin_right |EQ(REQUIRED)| self.right - margin.right as f64;
        self.constraints[6] = self.margin_top |EQ(REQUIRED)| self.top + margin.top as f64;
        self.constraints[7] = self.margin_bottom |EQ(REQUIRED)| self.bottom - margin.bottom as f64;
        self
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

    pub fn with_constraints<F: FnOnce(&mut Layout)->Vec<cassowary::Constraint>>(mut self, f: F) -> Self {
        let mut new = f(&mut self);
        self.constraints.append(&mut new);
        self
    }

    pub fn current<'a>(&'a self) -> Option<&'a Rect> {
        self.current.as_ref()
    }

    pub fn constraints<'a>(&'a self) -> &'a[cassowary::Constraint] {
        self.constraints.as_slice()
    }
}