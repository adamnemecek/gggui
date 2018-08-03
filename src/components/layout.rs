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
    edits: Vec<cassowary::Variable>,
    margin: Rect,
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

                margin_left |EQ(STRONG)| left,
                margin_right |EQ(STRONG)| right,
                margin_top |EQ(STRONG)| top,
                margin_bottom |EQ(STRONG)| bottom,
            ],
            edits: vec![],
            margin: Rect::zero(),
            current: None,

            left, right,
            top, bottom,
            center_x, center_y,
            width, height,
            margin_left, margin_right,
            margin_top, margin_bottom,
        }
    }

    pub fn as_editable(mut self, solver: &mut cassowary::Solver) -> Self {
        solver.add_edit_variable(self.left, STRONG).expect("unexpected edit variable error");
        solver.add_edit_variable(self.top, STRONG).expect("unexpected edit variable error");
        solver.add_edit_variable(self.right, STRONG).expect("unexpected edit variable error");
        solver.add_edit_variable(self.bottom, STRONG).expect("unexpected edit variable error");
        self.edits.push(self.left);
        self.edits.push(self.top);
        self.edits.push(self.right);
        self.edits.push(self.bottom);
        self
    }

    pub fn with_edit(mut self, edit: cassowary::Variable, solver: &mut cassowary::Solver) -> Self {
        solver.add_edit_variable(edit, STRONG).expect("unexpected edit variable error");
        self.edits.push(edit);
        self
    }

    pub fn with_detached_margin(mut self) -> Self {
        self.constraints.remove(4);
        self.constraints.remove(4);
        self.constraints.remove(4);
        self.constraints.remove(4);
        self
    }

    pub fn with_margins(mut self, margin: Rect) -> Self {
        self.constraints[4] = self.margin_left |EQ(REQUIRED)| self.left + margin.left as f64;
        self.constraints[5] = self.margin_right |EQ(REQUIRED)| self.right - margin.right as f64;
        self.constraints[6] = self.margin_top |EQ(REQUIRED)| self.top + margin.top as f64;
        self.constraints[7] = self.margin_bottom |EQ(REQUIRED)| self.bottom - margin.bottom as f64;
        self.margin = margin;
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

    pub fn edits<'a>(&'a self) -> &'a[cassowary::Variable] {
        self.edits.as_slice()
    }

    pub fn margin(&self) -> Rect {
        self.margin
    }
}

#[macro_export]
macro_rules! layout_rules {
    // Equations
    ($lookup:path, [ $a:expr ] = $($rest:tt)*) => {
        layout_rules!($lookup, [ $a |EQ(REQUIRED)| layout_rules!($lookup, [] $($rest)*)])
    };
    ($lookup:path, [ $a:expr ] >= $($rest:tt)*) => {
        layout_rules!($lookup, [ $a |GE(REQUIRED)| layout_rules!($lookup, [] $($rest)*)])
    };
    ($lookup:path, [ $a:expr ] <= $($rest:tt)*) => {
        layout_rules!($lookup, [ $a |LE(REQUIRED)| layout_rules!($lookup, [] $($rest)*)])
    };
    // Arithmetic
    ($lookup:path, [ $a:expr ] + $($rest:tt)*) => {
        layout_rules!($lookup, [ $a + layout_rules!($lookup, [] $($rest)*)])
    };
    ($lookup:path, [ $a:expr ] - $($rest:tt)*) => {
        layout_rules!($lookup, [ $a - layout_rules!($lookup, [] $($rest)*)])
    };
    ($lookup:path, [ $a:expr ] * $($rest:tt)*) => {
        layout_rules!($lookup, [ $a * layout_rules!($lookup, [] $($rest)*)])
    };
    ($lookup:path, [ $a:expr ] / $($rest:tt)*) => {
        layout_rules!($lookup, [ $a / layout_rules!($lookup, [] $($rest)*)])
    };
    // Parenthesis dereference
    ($lookup:path, [ $($stack:expr),* ] ($($rest:tt)*)) => {
        layout_rules!($lookup, [ $($stack ,)* ] $($rest)*)
    };
    // Variable lookup
    ($lookup:path, [ $($stack:expr),* ] $view:tt.$name:tt $($rest:tt)*) => {
        layout_rules!($lookup, [ $lookup(&format!("{0}.{1}", stringify!($view), stringify!($name))) $(, $stack)* ] $($rest)*)
    };
    // Fixed numbers
    ($lookup:path, [ $($stack:expr),* ] $num:tt $($rest:tt)*) => {
        layout_rules!($lookup, [ $num $(, $stack)* ] $($rest)*)
    };
    // Results
    ($lookup:path, [$result:expr]) => {
        $result
    };
    // Entry point
    ($ui:path, $($tokens:tt,)*) => {
        $ui.rules(|var| vec![$(layout_rules!(var, [] $tokens)),*])
    };        
}