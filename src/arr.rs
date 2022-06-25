use std::sync::Arc;
use std::sync::Mutex;
use std::ops::*;
use std::mem::*;

enum PerVec<T> {
    Arr(Vec<T>),
    Diff(usize, T, PerVecRef<T>),
    Pop(PerVecRef<T>),
    Push(T, PerVecRef<T>),
}

pub struct PerVecRef<T>(Arc<Mutex<PerVec<T>>>);

impl<T> PerVec<T> {
    fn unsafe_get_arr(&self) -> &Vec<T>{
        match self {
            PerVec::Arr(a) => &a,
            _ => panic!("get_arr failed")
        }
    }

    fn unsafe_get_arr_mut(&mut self) -> &mut Vec<T>{
        match self {
            PerVec::Arr(a) => a,
            _ => panic!("get_arr failed")
        }
    }
}

impl<T> Clone for PerVecRef<T> {
    
    fn clone(&self) -> Self { 
        PerVecRef(self.0.clone())
    }

}
impl<T> PerVecRef<T> {
    fn from_inner(t: PerVec<T>) -> Self {
        PerVecRef(
            Arc::new(
                Mutex::new(t)
            )
        )
    }

    pub fn new(t: Vec<T>) -> Self {
        PerVecRef::from_inner(PerVec::Arr(t))
    }

    fn reroot(&self, self_cell: &mut std::sync::MutexGuard<PerVec<T>>) {
        match self_cell.deref_mut() {
            PerVec::Arr(_) => {}
            PerVec::Diff(idx, val, t) => {
                // points to itself
                let next = replace(t, self.clone());
                let mut next_cell = next.0.lock().unwrap();
                next.reroot(&mut next_cell);

                let a = next_cell.deref_mut().unsafe_get_arr_mut();
                swap(&mut a[*idx], val);

                swap(self_cell.deref_mut(), next_cell.deref_mut());
            }
            PerVec::Push(_, _) => {
                let push = replace(self_cell.deref_mut(), PerVec::Pop(self.clone()));
                match push {
                    PerVec::Push(v, next) => {
                        let mut next_cell = next.0.lock().unwrap();
                        next.reroot(&mut next_cell);
                        
                        let a = next_cell.deref_mut().unsafe_get_arr_mut();
                        a.push(v);

                        swap(self_cell.deref_mut(), next_cell.deref_mut());
                    }
                    _ => unreachable!()
                }
            }
            PerVec::Pop(_) => {
                *self_cell.deref_mut() = match self_cell.deref_mut() {
                    PerVec::Pop(next) => {
                        let mut next_cell = next.0.lock().unwrap();
                        next.reroot(&mut next_cell);
                        
                        let a = next_cell.deref_mut().unsafe_get_arr_mut();
                        let to_push = a.pop().unwrap();
                        
                        replace(next_cell.deref_mut(), PerVec::Push(to_push, self.clone()))
                    }
                    _ => unreachable!()
                };
            }
        };
    }
}

impl<T: Clone> PerVecRef<T> {

    pub fn get(&self, i: usize) -> T {
        let mut self_cell = self.0.lock().unwrap();
        self.reroot(&mut self_cell);
        let a = self_cell.deref_mut().unsafe_get_arr_mut();
        a[i].clone()
    }

    pub fn set(&self, i: usize, v: T) -> Self {
        let ret = PerVecRef::from_inner(PerVec::Diff(i, v, self.clone()));
        ret.reroot(&mut ret.0.lock().unwrap());
        ret
    }

    pub fn push(&self, v: T) -> Self {
        let ret = PerVecRef::from_inner(PerVec::Push(v, self.clone()));
        ret.reroot(&mut ret.0.lock().unwrap());
        ret
    }

    pub fn len(&self) -> usize {
        let mut self_cell = self.0.lock().unwrap();
        self.reroot(&mut self_cell);
        self_cell.unsafe_get_arr().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

}

#[test]
fn test_basic() {
    let v = PerVecRef::new(vec![1, 2, 3]);
    let v1 = v.set(0, 3);
    let v2 = v1.set(1, 3);
    let v3 = v.set(1, 3);

    assert_eq!(v.get(0), 1);
    assert_eq!(v.get(1), 2);
    assert_eq!(v.get(2), 3);

    assert_eq!(v1.get(0), 3);
    assert_eq!(v1.get(1), 2);
    assert_eq!(v1.get(2), 3);

    assert_eq!(v2.get(0), 3);
    assert_eq!(v2.get(1), 3);
    assert_eq!(v2.get(2), 3);

    assert_eq!(v3.get(0), 1);
    assert_eq!(v3.get(1), 3);
    assert_eq!(v3.get(2), 3);

}

#[test]
fn test_push_pop() {
    let v = PerVecRef::new(vec![]);
    let v1 = v.push(0);
    assert_eq!(v1.get(0), 0);

    let v2 = v1.push(1);
    assert_eq!(v1.get(0), 0);
    assert_eq!(v2.get(0), 0);
    assert_eq!(v2.get(1), 1);

    let v3 = v.push(2);
    assert_eq!(v3.get(0), 2);

    assert_eq!(v1.get(0), 0);
    assert_eq!(v2.get(0), 0);
    assert_eq!(v2.get(1), 1);
}