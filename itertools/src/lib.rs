#![forbid(unsafe_code)]

use std::{cell::RefCell, collections::VecDeque, iter::from_fn, rc::Rc};

pub fn count() -> impl Iterator<Item = u64> {
    0..u64::MAX
}

pub fn cycle<T>(into_iter: T) -> impl Iterator<Item = T::Item>
where
    T: IntoIterator,
    T::Item: Clone,
{
    let mut content: Vec<T::Item> = vec![];
    let mut index = 0;
    let mut iter = into_iter.into_iter();
    let mut fatigue = false;
    from_fn(move || -> Option<T::Item> {
        if fatigue {
            if content.is_empty() {
                return None;
            }
            let answer = content[index].clone();
            index = (index + 1) % content.len();
            Some(answer)
        } else {
            match iter.next() {
                Some(value) => {
                    content.push(value.clone());
                    Some(value)
                }
                None => {
                    fatigue = true;
                    if content.is_empty() {
                        return None;
                    }
                    index = 1 % content.len();
                    Some(content[0].clone())
                }
            }
        }
    })
}

pub fn extract<T: IntoIterator>(
    into_iter: T,
    index: usize,
) -> (Option<T::Item>, impl Iterator<Item = T::Item>) {
    let mut content = VecDeque::<T::Item>::new();
    let mut iter = into_iter.into_iter();
    for _ in 0..index {
        if let Some(value) = iter.next() {
            content.push_back(value);
        }
    }
    (
        iter.next(),
        from_fn(move || {
            if let Some(value) = content.pop_front() {
                return Some(value);
            }
            iter.next()
        }),
    )
}

pub fn tee<T>(into_iter: T) -> (impl Iterator<Item = T::Item>, impl Iterator<Item = T::Item>)
where
    T: IntoIterator,
    T::Item: Clone,
{
    let rc = Rc::new(RefCell::new((
        VecDeque::<T::Item>::new(),
        into_iter.into_iter(),
        0,
        false,
    )));
    let mut cnt1 = 0;
    let rc2 = rc.clone();
    let mut cnt2 = 0;
    let first = move || {
        let mut resource = rc.borrow_mut();
        if cnt1 == resource.2 {
            if resource.3 {
                return None;
            }
            if let Some(value) = resource.1.next() {
                cnt1 += 1;
                resource.2 += 1;
                resource.0.push_back(value.clone());
                return Some(value);
            }
            resource.3 = true;
            return None;
        }
        cnt1 += 1;
        resource.0.pop_front()
    };
    let second = move || {
        let mut resource = rc2.borrow_mut();
        if cnt2 == resource.2 {
            if resource.3 {
                return None;
            }
            if let Some(value) = resource.1.next() {
                cnt2 += 1;
                resource.2 += 1;
                resource.0.push_back(value.clone());
                return Some(value);
            }
            resource.3 = true;
            return None;
        }
        cnt2 += 1;
        resource.0.pop_front()
    };
    (from_fn(first), from_fn(second))
}

pub fn group_by<T, F, V>(into_iter: T, mut f: F) -> impl Iterator<Item = (V, Vec<T::Item>)>
where
    T: IntoIterator,
    F: FnMut(&T::Item) -> V,
    V: Eq,
{
    let mut iter = into_iter.into_iter();
    let mut holdback = VecDeque::<(V, T::Item)>::new();
    from_fn(move || {
        if holdback.is_empty() {
            match iter.next() {
                None => return None,
                Some(value) => holdback.push_back((f(&value), value)),
            }
        }
        let (flag, first) = holdback.pop_front().unwrap();
        let mut answer = vec![first];
        loop {
            if let Some(value) = iter.next() {
                let cur_flag = f(&value);
                if flag == cur_flag {
                    answer.push(value);
                    continue;
                } else {
                    holdback.push_back((cur_flag, value));
                    break;
                }
            }
            break;
        }
        Some((flag, answer))
    })
}
