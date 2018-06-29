use super::*;

pub trait Entry {
    fn eval<'a>(self: Box<Self>, context: &mut Context<'a>);
}

impl<T: 'static + Widget> Entry for (&'static str, T) {
    fn eval<'a>(self: Box<Self>, context: &mut Context<'a>) {
        let x = *self;
        context.add(x.0, x.1);
    }
}

impl<T: 'static + Widget, F: FnOnce(T::Result)> Entry for (&'static str, T, F) {
    fn eval<'a>(self: Box<Self>, context: &mut Context<'a>) {
        let x = *self;
        x.2(context.add(x.0, x.1).result);
    }
}

impl<'a> Context<'a> {
    fn add_entries(&mut self, x: Vec<Box<Entry>>) {
        for i in x {
            i.eval(self);
        }
    }
}

impl<T: 'static + Widget> Entry for (&'static str, T, Vec<Box<Entry>>) {
    fn eval<'a>(self: Box<Self>, context: &mut Context<'a>) {
        let x = *self;
        context.add(x.0, x.1).context.add_entries(x.2);
    }
}
