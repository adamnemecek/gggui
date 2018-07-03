use super::*;
use primitive::Primitive;

pub mod background_render;
pub mod drawing_render;
pub mod clickable_event;
pub mod content_clip;

pub use self::background_render::BackgroundRenderSystem;
pub use self::drawing_render::DrawingRenderSystem;
pub use self::clickable_event::ClickableEventSystem;
pub use self::content_clip::ContentPushClipSystem;
pub use self::content_clip::ContentPopClipSystem;
pub use self::content_clip::new_clip_system;

pub trait System<C> {
    type Components: SystemComponents;
    fn run(&self, sys_context: &mut C, args: Self::Components);
}

pub trait SystemDispatch<C> {
    fn run_for(&self, sys_context: &mut C, id: dag::Id, world: &Ui) -> Result<(), ()>;
}

pub trait SystemComponents: Sized {
    fn fetch(world: &Ui, id: dag::Id) -> Result<Self, ()>;
}

macro_rules! impl_components {
    ($($x:ident),*) => (
        impl<$($x:Fetch),*> SystemComponents for ($($x),*) {
            #[allow(unused_parens)]
            fn fetch(world: &Ui, id: dag::Id) -> Result<Self, ()> {
                Ok((
                    $($x::fetch(world, id)?),*
                ))
            }
        }
    )
}

impl<C, T: System<C>> SystemDispatch<C> for T {
    fn run_for(&self, sys_context: &mut C, id: dag::Id, world: &Ui) -> Result<(), ()> {
        Ok(self.run(sys_context, T::Components::fetch(world, id)?))
    }
}

impl_components!(A);
impl_components!(A,B);
impl_components!(A,B,C);
impl_components!(A,B,C,D);
impl_components!(A,B,C,D,E);
impl_components!(A,B,C,D,E,F);
impl_components!(A,B,C,D,E,F,G);
impl_components!(A,B,C,D,E,F,G,H);
impl_components!(A,B,C,D,E,F,G,H,I);
impl_components!(A,B,C,D,E,F,G,H,I,J);
impl_components!(A,B,C,D,E,F,G,H,I,J,K);
impl_components!(A,B,C,D,E,F,G,H,I,J,K,L);
impl_components!(A,B,C,D,E,F,G,H,I,J,K,L,M);
impl_components!(A,B,C,D,E,F,G,H,I,J,K,L,M,N);
impl_components!(A,B,C,D,E,F,G,H,I,J,K,L,M,N,O);
impl_components!(A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P);
impl_components!(A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q);
impl_components!(A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R);
impl_components!(A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S);
impl_components!(A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T);
impl_components!(A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U);
impl_components!(A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V);
impl_components!(A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W);
impl_components!(A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X);
impl_components!(A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y);
impl_components!(A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z);