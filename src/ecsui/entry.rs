use super::*;

pub trait EntryBase<'c> {
    fn eval(self, &mut Context<'c>);
}

pub enum Entry<'a, 'c, T: 'static + Widget> {
    Leaf(&'a str, T),
    Handler(&'a str, T, Box<FnOnce(T::Result)>),
    Node(&'a str, T, Vec<Box<EntryBase<'c>>>),
}

impl<'a, 'c, T: 'static + Widget> EntryBase<'c> for Entry<'a, 'c, T> {
    fn eval(self, context: &mut Context<'c>) {
        match self {
            Entry::Leaf(id, x) => {
                context.add(id, x);
            },
            Entry::Handler(id, x, l) => {
                l(context.add(id, x).result);
            },
            Entry::Node(id, x, y) => {
                let sub = &mut context.add(id, x).context;
                for i in y {
                    i.eval(&mut sub);
                }
            },
        }
    }
}