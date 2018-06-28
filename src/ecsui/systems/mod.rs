use super::*;
use primitive::Primitive;

pub struct SystemContext {
    pub drawlist: Vec<Primitive>,
}

pub trait System {
    type Components;
    fn run(&self, sys_context: &mut SystemContext, args: Self::Components);
}

pub trait SystemDispatch {
    fn run_for(&self, sys_context: &mut SystemContext, id: dag::Id, world: &Ui) -> Result<(), ()>;
}

macro_rules! impl_dispatch {
    ($($x:ident),*) => (
        impl<$($x:'static+Clone),*> SystemDispatch for System<Components=($(FetchComponent<$x>),*)> 
        {
            #[allow(unused_parens)]
            fn run_for(&self, s: &mut SystemContext, id: dag::Id, world: &Ui) -> Result<(), ()> {
                Ok(self.run(s, ($(
                    world.component::<$x>(id).ok_or(())?),*
                )))
            }
        }
    )
}

impl_dispatch!(A);
impl_dispatch!(A,B);
impl_dispatch!(A,B,C);
impl_dispatch!(A,B,C,D);
impl_dispatch!(A,B,C,D,E);
impl_dispatch!(A,B,C,D,E,F);
impl_dispatch!(A,B,C,D,E,F,G);
impl_dispatch!(A,B,C,D,E,F,G,H);
impl_dispatch!(A,B,C,D,E,F,G,H,I);
impl_dispatch!(A,B,C,D,E,F,G,H,I,J);
impl_dispatch!(A,B,C,D,E,F,G,H,I,J,K);