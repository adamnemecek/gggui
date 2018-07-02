use super::*;

pub struct ContentPushClipSystem { }

pub struct ContentPopClipSystem { }

impl System<Vec<Primitive>> for ContentPushClipSystem {
    type Components = (
        FetchComponent<Clipper>
    );
    fn run(&self, drawlist: &mut Vec<Primitive>, clipper: Self::Components) {
        drawlist.push(Primitive::PushClip(clipper.borrow().rect));
    }
}

impl System<Vec<Primitive>> for ContentPopClipSystem {
    type Components = (
        FetchComponent<Clipper>
    );
    fn run(&self, drawlist: &mut Vec<Primitive>, _clipper: Self::Components) {
        drawlist.push(Primitive::PopClip);
    }
}