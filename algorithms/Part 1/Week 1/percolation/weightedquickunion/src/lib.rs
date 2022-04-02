pub struct WeightedQuickUnionUF {
    parent: Vec<usize>,
    size: Vec<usize>,
    count: usize,
}

impl Default for WeightedQuickUnionUF {
    fn default() -> Self {
        WeightedQuickUnionUF {
            parent: Vec::new(),
            size: vec![],
            count: 0,
        }
    }
}

impl WeightedQuickUnionUF {
    pub fn new(count: usize) -> Self {
        WeightedQuickUnionUF {
            parent: Vec::from_iter(0..count),
            size: vec![1; count],
            count,
        }
    }
    pub fn count(&self) -> usize {
        self.count
    }
    pub fn find(&self, mut node: usize) -> usize {
        if node >= self.parent.len() {
            panic!("index {} is not between 0 and {}", node, self.count - 1);
        }
        while node != self.parent[node] {
            node = self.parent[node];
        }
        node
    }
    #[deprecated]
    pub fn connected(&self, node1: usize, node2: usize) -> bool {
        self.find(node1) == self.find(node2)
    }
    pub fn union(&mut self, node1: usize, node2: usize) {
        let root1 = self.find(node1);
        let root2 = self.find(node2);
        if root1 == root2 {
            return;
        }
        if self.size[root1] < self.size[root2] {
            self.parent[root1] = root2;
            self.size[root2] += self.size[root1];
        } else {
            self.parent[root2] = root1;
            self.size[root1] += self.size[root2];
        }
        self.count -= 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::WeightedQuickUnionUF;

    #[test]
    fn weightedquickunion_works() {
        let count = 2;
        let mut uf = WeightedQuickUnionUF::new(count);
        if uf.find(0) != uf.find(1) {
            uf.union(0, 1);
        }
        assert_eq!(uf.find(0), uf.find(1));
        assert_eq!(uf.count(), 1)
    }
}
