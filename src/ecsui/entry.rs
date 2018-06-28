use super::*;

pub trait Entry<'c> {
    fn eval(self, &'c mut Context<'c>);
}

impl<'c, T: 'static + Widget> Entry<'c> for (&'static str, T) {
    fn eval(self, context: &'c mut Context<'c>) {
        context.add(self.0, self.1);
    }
}

impl<'c, T: 'static + Widget, F: FnOnce(T::Result)> Entry<'c> for (&'static str, T, F) {
    fn eval(self, context: &'c mut Context<'c>) {
        self.2(context.add(self.0, self.1).result);
    }
}

fn dispatch<'c>(_context: &'c mut Context<'c>, _entries: Vec<Box<Entry<'c>>>) {
    _entries[0].eval(_context);
    _entries[1].eval(_context);
    _entries[2].eval(_context);
    //for x in entries {
    //    x.eval(context);
    //}
}

impl<'c: 'd, 'd, T: 'static + Widget> Entry<'c> for (&'static str, T, Vec<Box<Entry<'d>>>) {
    fn eval(self, context: &'c mut Context<'c>) {
        let mut result = context.add(self.0, self.1);
        dispatch(&mut result.context, self.2);
    }
}