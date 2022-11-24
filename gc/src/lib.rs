#![forbid(unsafe_code)]

pub use gc_derive::Scan;

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    marker::PhantomData,
    ops::Deref,
    rc::{Rc, Weak},
};

////////////////////////////////////////////////////////////////////////////////

pub struct Gc<T> {
    pub weak: Weak<T>,
}

impl<T> Gc<T> {
    pub fn get_address(&self) -> usize {
        self.weak.as_ptr() as usize
    }
}
impl<T> Clone for Gc<T> {
    fn clone(&self) -> Self {
        Self {
            weak: self.weak.clone(),
        }
    }
}

impl<T> Gc<T> {
    pub fn borrow(&self) -> GcRef<'_, T> {
        GcRef {
            rc: self.weak.upgrade().unwrap(),
            lifetime: PhantomData::<&'_ Gc<T>>,
        }
    }
}

pub struct GcRef<'a, T> {
    rc: Rc<T>,
    lifetime: PhantomData<&'a Gc<T>>,
}

impl<'a, T> Deref for GcRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.rc
    }
}

////////////////////////////////////////////////////////////////////////////////

pub trait Scan {
    fn get_allocations(&self) -> Vec<usize>;
}

impl<T: Scan + 'static> Scan for Gc<T> {
    fn get_allocations(&self) -> Vec<usize> {
        vec![self.get_address()]
    }
}

impl<T: Scan + 'static> Scan for Vec<T> {
    fn get_allocations(&self) -> Vec<usize> {
        let mut answer: Vec<usize> = vec![];
        for item in self.iter() {
            answer.append(&mut item.get_allocations());
        }
        answer
    }
}

impl<T: Scan + 'static> Scan for Option<T> {
    fn get_allocations(&self) -> Vec<usize> {
        if let Some(resource) = self.as_ref() {
            return resource.get_allocations();
        }
        vec![]
    }
}

impl<T: Scan + 'static> Scan for RefCell<T> {
    fn get_allocations(&self) -> Vec<usize> {
        self.borrow().get_allocations()
    }
}

impl Scan for i32 {
    fn get_allocations(&self) -> Vec<usize> {
        vec![]
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Arena {
    allocations: Vec<Rc<dyn Scan + 'static>>,
}

impl Arena {
    pub fn new() -> Self {
        Self {
            allocations: Vec::<Rc<dyn Scan + 'static>>::new(),
        }
    }

    pub fn allocation_count(&self) -> usize {
        self.allocations.len()
    }

    pub fn alloc<T: Scan + 'static>(&mut self, obj: T) -> Gc<T> {
        let rc: Rc<T> = Rc::new(obj);
        let weak = Rc::<T>::downgrade(&rc);
        self.allocations.push(rc);
        Gc { weak }
    }

    pub fn sweep(&mut self) {
        let index: HashMap<usize, usize> = (0..self.allocations.len())
            .map(|i| (Rc::as_ptr(&self.allocations[i]) as *const u8 as usize, i))
            .collect();

        let mut cnt = vec![0; self.allocations.len()];
        let graph: Vec<Vec<usize>> = self
            .allocations
            .iter()
            .map(|alloc| {
                alloc
                    .get_allocations()
                    .into_iter()
                    .map(|x| {
                        let ans = index[&x];
                        cnt[ans] += 1;
                        ans
                    })
                    .collect()
            })
            .collect();

        let mut marked = HashSet::<usize>::new();
        for (i, links) in cnt.iter().enumerate() {
            if Rc::weak_count(&self.allocations[i]) <= *links {
                continue;
            }
            Self::mark_all(i, &mut marked, &graph);
        }
        let mut anchor = 0;
        for i in 0..self.allocations.len() {
            if marked.contains(&i) {
                if i > anchor {
                    self.allocations.swap(anchor, i);
                }
                anchor += 1;
            }
        }
        self.allocations.truncate(anchor);
    }

    fn mark_all(root_addr: usize, marked: &mut HashSet<usize>, graph: &Vec<Vec<usize>>) {
        //println!("Marking {}", root_addr);
        //println!("{:?}", graph[root_addr]);
        if !marked.insert(root_addr) {
            return;
        }
        for u in graph[root_addr].iter() {
            Self::mark_all(*u, marked, graph);
        }
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}
