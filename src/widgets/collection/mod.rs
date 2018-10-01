use cassowary::strength::*;
use cassowary::WeightedRelation::*;
use super::*;

mod linear;
pub use self::linear::*;

pub trait Layouter {
    fn header(&self,
              _index: usize,
              _item: &Layout,
              _parent: &Layout
    ) -> Vec<cassowary::Constraint> {
        vec![]
    }

    fn item(&self,
            _index: usize,
            _item: &Layout,
            _parent: &Layout,
            _previous: &Layout
    ) -> Vec<cassowary::Constraint> {
        vec![]
    }

    fn footer(&self,
              _index: usize,
              _item: &Layout,
              _parent: &Layout
    ) -> Vec<cassowary::Constraint> {
        vec![]
    }
}

pub struct Collection<L: Layouter> {
    layout: L,
}

impl<L: Layouter> Collection<L> {
    pub fn new(layout: L) -> Self {
        Self { layout }
    }
}

#[derive(Clone)]
struct CacheItem {
    id: dag::Id,
    first_last: (bool, bool),
    prev: Option<dag::Id>,
    cons: Vec<cassowary::Constraint>,
}

impl<L: Layouter> WidgetBase for Collection<L> {
    fn create(&mut self, id: dag::Id, world: &mut Ui, _style: &Style) {
        // layout component
        world.create_component(id, Layout::new());

        // list of id's for the stored items.
        world.create_component(id, Vec::<CacheItem>::new());
    }

    fn update(&mut self, id: dag::Id, world: &mut Ui, _style: &Style, input: Option<Rect>) -> Option<Rect> {
        let mut previous = None;
        let mut previous_layout: Option<FetchComponent<Layout>> = None;

        let mut cache: FetchComponent<Vec<CacheItem>> = world.component(id).unwrap();
        let mut cache = cache.borrow_mut();

        let mut count = 0;

        let parent: FetchComponent<Layout> = world.component(id).unwrap();

        let mut old_constraints = vec![];
        let mut new_constraints = vec![];

        // perform layout resolve for invalidated items
        for (i, (is_first, is_last, &id)) in world.children().identify_first_last().enumerate() {

            let item: FetchComponent<Layout> = world.component(id).unwrap();

            let is_cached = cache.get(i).map(|x| x.id == id &&
                x.prev == previous &&
                x.first_last == (is_first, is_last)
            ).unwrap_or(false);

            if !is_cached {
                // queue old constraints for removal
                cache.get(i).map(|x| {
                    for c in x.cons.iter() {
                        old_constraints.push(c.clone());
                    }
                });

                // generate new constraints
                let constraints = {
                    let mut x = vec![];
                    if is_first {
                        x.append(&mut self.layout.header(i,
                            &*item.borrow(),
                            &*parent.borrow()
                        ));
                    }
                    if is_last {
                        x.append(&mut self.layout.footer(i,
                            &*item.borrow(),
                            &*parent.borrow()
                        ))
                    }
                    previous_layout.as_ref().map(|previous| {
                        x.append(&mut self.layout.item(i,
                            &*item.borrow(),
                            &*parent.borrow(),
                            &*previous.borrow()
                        ));
                    });
                    x
                };

                // queue the new constraints
                for c in constraints.iter() {
                    new_constraints.push(c.clone());
                }

                // update cache
                if i >= cache.len() {
                    cache.push(CacheItem {
                        id: id,
                        first_last: (is_first, is_last),
                        prev: previous,
                        cons: constraints
                    });
                } else {
                    cache[i] = CacheItem {
                        id: id,
                        first_last: (is_first, is_last),
                        prev: previous,
                        cons: constraints
                    };
                }
            }

            previous = Some(id);
            previous_layout = Some(item);
            count += 1;
        }

        // cleanup old constraints
        for c in old_constraints {
            world.layout_solver.remove_constraint(&c).ok();
        }
        for o in cache.split_off(count) {
            for c in o.cons.iter() {
                world.layout_solver.remove_constraint(c).ok();
            }
        }

        // introduce new constraints
        world.layout_solver.add_constraints(new_constraints.iter()).ok();

        let parent = parent.borrow();
        parent.current().and_then(|content| input.and_then(|ir| ir.intersect(&content)))
    }
}

impl<L: Layouter> Widget for Collection<L> {
    type Result = ();

    fn result(&mut self, _: dag::Id) -> Self::Result { }
}
