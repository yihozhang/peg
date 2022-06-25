mod arr;
use arr::PerVecRef;

struct PerUnionFind {
    parent: PerVecRef<usize>,
}

impl PerUnionFind {
    pub fn new() -> Self {
        Self {
            parent: PerVecRef::new(vec![])
        }
    }

    pub fn add(&self) -> (usize, Self) {
        let new_parent = self.parent.push(self.parent.len());
        (self.parent.len(), (PerUnionFind { parent: new_parent }))
    }

    pub fn find(&mut self, x: usize) -> usize {
        let fx = self.parent.get(x);
        if fx != x {
            let fx = self.find(fx);
            self.parent = self.parent.set(x, fx);
            fx
        } else {
            fx
        }
    } 

    pub fn merge(&mut self, x: usize, y: usize) -> Self {
        let fx = self.find(x);
        let fy = self.find(y);
        let new_parent = self.parent.set(fx, fy);
        PerUnionFind { parent: new_parent }
    }
}

#[test]
fn union_find_basic() {
    let uf = PerUnionFind::new();
    let (x, uf) = uf.add();
    let (y, mut uf) = uf.add();
    assert_eq!(uf.find(x), x);
    assert_eq!(uf.find(y), y);
    assert_ne!(uf.find(x), uf.find(y));
    let mut new_uf = uf.merge(x, y);
    assert_eq!(new_uf.find(x), new_uf.find(y));

    // old version still valid
    assert_eq!(uf.find(x), x);
    assert_eq!(uf.find(y), y);
    assert_ne!(uf.find(x), uf.find(y));
}