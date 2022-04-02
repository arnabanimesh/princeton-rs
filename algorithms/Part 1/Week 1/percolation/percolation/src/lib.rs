use weightedquickunion::*;
pub mod gui;

const CLOSE: u8 = 0;
const OPEN: u8 = 1;
const TOP: u8 = 2;
const BOTTOM: u8 = 4;
const PERCOLATE: u8 = TOP | BOTTOM;

pub struct Percolation {
    length: usize,
    nopen: usize,
    id: WeightedQuickUnionUF,
    open: Vec<u8>,
    percolates: bool,
}

impl Default for Percolation {
    fn default() -> Self {
        Percolation {
            length: 0,
            nopen: 0,
            id: WeightedQuickUnionUF::default(),
            open: Vec::new(),
            percolates: false,
        }
    }
}

impl Percolation {
    pub fn new(n: usize) -> Self {
        let size = n * n;
        Percolation {
            length: n,
            nopen: 0,
            id: WeightedQuickUnionUF::new(size),
            open: vec![0; size],
            percolates: false,
        }
    }
    fn adjust(&self, row: usize, col: usize) -> (usize, usize) {
        if row > self.length || col > self.length || row < 1 || col < 1 {
            panic!("Invalid (row, col): ({},{})", row, col);
        }
        (row - 1, col - 1)
    }
    fn index(&self, coordinates: (usize, usize)) -> usize {
        coordinates.0 * self.length + coordinates.1
    }
    pub fn connect(&mut self, idx: usize, idxnear: usize) -> u8 {
        let findnear: usize = self.id.find(idxnear);
        if self.open[findnear] != CLOSE {
            self.id.union(idx, idxnear);
            return self.open[findnear];
        }
        CLOSE
    }
    pub fn open(&mut self, row: usize, col: usize) {
        let (row, col) = self.adjust(row, col);
        let index = self.index((row, col));
        if self.open[index] & OPEN == 0 {
            let mut status = OPEN;
            if col >= 1 {
                status |= self.connect(index, index - 1);
            }
            if col + 1 < self.length {
                status |= self.connect(index, index + 1);
            }
            if row >= 1 {
                status |= self.connect(index, index - self.length);
            }
            if row + 1 < self.length {
                status |= self.connect(index, index + self.length);
            }
            let f = self.id.find(index);
            if row == 0 {
                self.open[f] |= TOP;
            }
            if row == self.length - 1 {
                self.open[f] |= BOTTOM;
            }
            self.open[f] |= status;
            if (self.open[f] & PERCOLATE) == PERCOLATE {
                self.percolates = true;
            }
            self.open[index] = self.open[f];
            self.nopen += 1;
        }
    }
    pub fn is_open(&self, row: usize, col: usize) -> bool {
        if self.open[self.index(self.adjust(row, col))] != 0 {
            return true;
        }
        false
    }
    pub fn is_full(&self, row: usize, col: usize) -> bool {
        let index = self.index(self.adjust(row, col));
        if self.open[index] != 0 {
            return (self.open[self.id.find(index)] & TOP) == TOP;
        }
        false
    }
    pub fn number_of_open_sites(&self) -> usize {
        self.nopen
    }
    pub fn percolates(&self) -> bool {
        self.percolates
    }
}

#[cfg(test)]
mod tests {

    use crate::Percolation;

    #[test]
    fn percolation_works() {
        let mut id = Percolation::new(3);
        id.open(1, 3);
        id.open(2, 3);
        assert!(!id.percolates());
        id.open(3, 3);
        assert!(id.percolates());
        id.open(3, 1);
        assert!(!id.is_full(3, 1));
    }
}
