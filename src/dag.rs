use std::mem::replace;
use std::collections::HashMap;

pub type Id = (usize, usize);

pub struct Item {
    pub id: Id,
    pub used: usize,
    pub subs: Option<Tree>,
}

pub struct Tree {
    pub ids: HashMap<String, Item>,
    pub ord: Vec<Id>,
}

pub struct FreeList {
    recently_freed: Vec<Id>,
    free: Vec<Id>,
    next: usize,
}

impl FreeList {
    pub fn new() -> Self {
        Self {
            free: vec![],
            recently_freed: vec![],
            next: 0
        }
    }

    pub fn push(&mut self, (x, gen): Id) {
        self.free.push((x, gen+1));
        self.recently_freed.push((x, gen));
    }

    pub fn pop(&mut self) -> Id {
        self.free.pop().unwrap_or_else(|| {
            self.next += 1;
            (self.next, 1)
        })
    }

    pub fn fetch_recently_freed(&mut self) -> Vec<Id> {
        replace(&mut self.recently_freed, vec![])
    }

    pub fn len(&self) -> usize {
        self.next as usize + 1
    }
}

impl Tree {
    pub fn new() -> Self {
        Tree {
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
                free_list.push(v.id);
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