use std::mem::replace;
use std::collections::HashMap;
use cassowary;

pub type Id = (usize, usize);

pub struct Item {
    pub id: Id,
    pub used: usize,
    pub subs: Option<Tree>,
}

pub struct Tree {
    pub vars: HashMap<String, cassowary::Variable>,
    pub rules: Vec<cassowary::Constraint>,
    pub ids: HashMap<String, Item>,
    pub ord: Vec<Id>,
}

pub struct FreeList {
    recently_freed_ids: Vec<Id>,
    recently_freed_constraints: Vec<cassowary::Constraint>,
    free: Vec<Id>,
    next: usize,
}

impl FreeList {
    pub fn new() -> Self {
        Self {
            free: vec![],
            recently_freed_ids: vec![],
            recently_freed_constraints: vec![],
            next: 0
        }
    }

    pub fn push(&mut self, (x, gen): Id, mut constraints: Vec<cassowary::Constraint>) {
        self.free.push((x, gen+1));
        self.recently_freed_ids.push((x, gen));
        self.recently_freed_constraints.append(&mut constraints);
    }

    pub fn pop(&mut self) -> Id {
        self.free.pop().unwrap_or_else(|| {
            self.next += 1;
            (self.next, 1)
        })
    }

    pub fn fetch_recently_freed_ids(&mut self) -> Vec<Id> {
        replace(&mut self.recently_freed_ids, vec![])
    }

    pub fn fetch_recently_freed_constraints(&mut self) -> Vec<cassowary::Constraint> {
        replace(&mut self.recently_freed_constraints, vec![])
    }

    pub fn len(&self) -> usize {
        self.next as usize + 1
    }
}

impl Tree {
    pub fn new() -> Self {
        Tree {
            vars: HashMap::new(),
            rules: Vec::new(),
            ids: HashMap::new(),
            ord: Vec::new(),
        }
    }

    pub fn cleanup(&mut self, before: usize, free_list: &mut FreeList) {
        self.ids.retain(|_, ref mut v| {
            v.subs.as_mut().map(|sub| sub.cleanup(before, free_list));
            if v.used >= before {
                true
            } else { 
                free_list.push(v.id, v.subs
                    .as_mut()
                    .map(|t| replace(&mut t.rules, vec![]))
                    .unwrap_or(vec![])
                );
                false
            }
        });
    }

    pub fn id(&mut self, name: &str, free_list: &mut FreeList) -> Id {
        match self.ids.get(name) {
            Some(item) => return item.id,
            None => (),
        }

        self.ids.entry(name.to_string()).or_insert(Item {
            id: free_list.pop(),
            used: 0,
            subs: None,
        }).id
    }

    pub fn item<'a>(&'a mut self, name: &str, free_list: &mut FreeList) -> &'a mut Item {
        if self.ids.contains_key(name) {
            self.ids.get_mut(name).unwrap()
        } else {
            self.ids.entry(name.to_string()).or_insert(Item {
                id: free_list.pop(),
                used: 0,
                subs: None,
            })
        }        
    }
}