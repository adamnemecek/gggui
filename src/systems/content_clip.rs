use super::*;

pub struct ContentPushClipSystem {
    current: Rc<RefCell<Vec<Rect>>>,
}

pub struct ContentPopClipSystem {
    current: Rc<RefCell<Vec<Rect>>>,
}

pub fn new_clip_system() -> (ContentPushClipSystem, ContentPopClipSystem) {
    let current = Rc::new(RefCell::new(Vec::new()));
    let push = ContentPushClipSystem{ current: current.clone() };
    let pop = ContentPopClipSystem{ current: current.clone() };
    
    (push, pop)
}

impl System<Vec<Primitive>> for ContentPushClipSystem {
    type Components = (
        FetchComponent<Clipper>
    );
    fn run(&self, drawlist: &mut Vec<Primitive>, clipper: Self::Components) {
        let mut stack = self.current.borrow_mut();
        let clipper = clipper.borrow();
        if stack.len() > 0 && clipper.intersect {
            let rect = clipper.rect.intersect(&stack[stack.len()-1]).unwrap_or(Rect::from_wh(0.0, 0.0));
            stack.push(rect);
        } else {
            stack.push(clipper.rect);
        }

        drawlist.push(Primitive::PushClip(stack[stack.len()-1]));
    }
}

impl System<Vec<Primitive>> for ContentPopClipSystem {
    type Components = (
        FetchComponent<Clipper>
    );
    fn run(&self, drawlist: &mut Vec<Primitive>, _clipper: Self::Components) {
        let mut stack = self.current.borrow_mut();
        stack.pop();
        drawlist.push(Primitive::PopClip);
    }
}