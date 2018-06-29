use super::*;

#[derive(Clone,Copy,Debug)]
pub enum Clickable {
    Idle,
    Hovering,
    Clicked(bool),
    Released(bool),
}