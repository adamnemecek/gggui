use super::*;

pub trait Entry {
    fn eval<'a>(&self, context: &mut Context<'a>);
}

impl<T: 'static + Widget + Clone> Entry for (&'static str, T) {
    fn eval<'a>(&self, context: &mut Context<'a>) {
        context.add(self.0, self.1.clone());
    }
}

impl<T: 'static + Widget + Clone, F: Fn(T::Result)> Entry for (&'static str, T, F) {
    fn eval<'a>(&self, context: &mut Context<'a>) {
        self.2(context.add(self.0, self.1.clone()).result);
    }
}

impl<'a> Context<'a> {
    fn add_entries(&mut self, x: &'a Vec<&'a Entry>) {
        for i in x {
            i.eval(self);
        }
    }
}

impl<'e, T: 'static + Widget + Clone> Entry for (&'static str, T, Vec<&'e Entry>) {
    fn eval<'a>(&self, context: &mut Context<'a>) {
        context.add(self.0, self.1.clone()).context.add_entries(&self.2);
    }
}
