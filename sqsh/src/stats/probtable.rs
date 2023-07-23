use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

pub trait Countable: Eq + Hash + Clone + Debug {}
impl<T> Countable for T where T: Eq + Hash + Clone + Debug {}

type RefNode<T> = Rc<RefCell<Count<T>>>;

#[derive(Default, Debug)]
struct Count<T> {
    val: T,
    count: usize,
    rank: usize,
}

impl<T: Countable> Count<T> {
    fn new(val: T, rank: usize) -> Self {
        Count {
            val,
            count: 1,
            rank,
        }
    }
    fn inc(&mut self) {
        self.count += 1;
    }
}

#[derive(Default)]
pub struct ProbTable<T> {
    hm: HashMap<T, RefNode<T>>,
    sorted_vec: Vec<RefNode<T>>,
}

impl<T: Countable + Debug> Debug for ProbTable<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProbTable[")?;
        for node in &self.sorted_vec {
            let c = node.borrow().count;
            let v = &node.borrow().val;
            write!(f, "({:?}:{})", v, c)?;
        }
        write!(f, "]")
    }
}

/// A sorted probability table implemented using a `HashMap` and and a `Vector`.
///
/// this table counts the occurrences of an element and saves it in a sorted
/// manner.
///
/// # Examples
///
/// ```
/// use sqsh::stats::ProbTable;
///
/// // Initialize a table for counting `u8` values
/// let mut table = ProbTable::<u8>::new();
///
/// table.insert(42);
/// table.insert(42);
/// table.insert(31);
///
/// // Get rank of a single value
/// assert_eq!(table.rank(&42), Some(0));
/// assert_eq!(table.rank(&4), None);
///
/// // Get count of a single value
/// assert_eq!(table.count(&42), Some(2));
/// assert_eq!(table.count(&9), None);
///
/// // The query can also be reversed. Such that an element of `T` is returned
/// // based on its position in the table
/// assert_eq!(table.position(0), Some(42));
/// assert_eq!(table.position(9), None);
/// ```
#[allow(dead_code)]
impl<T: Countable> ProbTable<T> {
    /// Create a new `ProbTable`
    ///
    /// # Examples
    ///
    /// ```
    /// use sqsh::stats::ProbTable;
    ///
    /// let table = ProbTable::<u8>::new();
    /// ```
    pub fn new() -> Self {
        ProbTable {
            hm: HashMap::<T, RefNode<T>>::new(),
            sorted_vec: Vec::<RefNode<T>>::new(),
        }
    }

    /// Create a new `ProbTable` with certain symbol capacity
    ///
    /// # Examples
    ///
    /// ```
    /// use sqsh::stats::ProbTable;
    ///
    /// let table = ProbTable::<u8>::with_capacity(13);
    /// assert_eq!(table.capacity(), 13)
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        ProbTable {
            hm: HashMap::<T, RefNode<T>>::with_capacity(capacity),
            sorted_vec: Vec::<RefNode<T>>::with_capacity(capacity),
        }
    }

    /// Feed table with several values from a vector
    ///
    /// # Examples
    ///
    /// ```
    /// use sqsh::stats::ProbTable;
    ///
    /// let mut table = ProbTable::<u8>::new();
    /// let data = vec![3, 4, 3, 3, 3, 3, 4, 5, 8];
    /// table.feed(&data);
    ///
    /// assert_eq!(table.members(), 4);
    /// ```
    pub fn feed<R: AsRef<[T]>>(&mut self, vals: R) {
        let vals = vals.as_ref();
        let mut local = Vec::new();
        local.extend(vals.iter().cloned());
        for val in local {
            self.insert(val);
        }
    }

    /// Insert a single new data point to table
    ///
    /// # Examples
    ///
    /// ```
    /// use sqsh::stats::ProbTable;
    ///
    /// let mut table = ProbTable::<u8>::new();
    /// table.insert(4);
    ///
    /// assert_eq!(table.members(), 1);
    /// ```
    pub fn insert(&mut self, val: T) -> usize {
        // println!("Inserting {:?} to {:?}", val, self);
        let r = match self.hm.get_mut(&val) {
            Some(node) => {
                node.borrow_mut().inc();
                let ix = node.borrow().rank;
                self.renormalize(ix)
            }
            None => {
                let rank = self.sorted_vec.len();
                let node = Rc::new(RefCell::new(Count::new(val.clone(), rank)));
                self.hm.insert(val, node.clone());
                self.sorted_vec.push(node);
                self.renormalize(rank)
            }
        };
        // println!("Inserted {:?}", self);
        debug_assert!(self.is_coherent());
        r
    }

    /// Get rank of single element in table
    ///
    /// # Examples
    ///
    /// ```
    /// use sqsh::stats::ProbTable;
    ///
    /// let mut table = ProbTable::<u8>::new();
    /// let data = vec![3, 4, 3, 3, 3, 3, 4, 5, 8];
    /// table.feed(&data);
    ///
    /// assert_eq!(table.rank(&3), Some(0));
    /// assert_eq!(table.rank(&9), None);
    /// ```
    pub fn rank(&self, val: &T) -> Option<usize> {
        match self.hm.get(val) {
            Some(node) => {
                let ix = node.borrow().rank;
                Some(ix)
            }
            None => None,
        }
    }

    /// Get element at position of table
    ///
    /// # Examples
    ///
    /// ```
    /// use sqsh::stats::ProbTable;
    ///
    /// let mut table = ProbTable::<u8>::new();
    /// let data = vec![3, 4, 3, 3, 3, 3, 4, 5, 8];
    /// table.feed(&data);
    ///
    /// assert_eq!(table.position(0), Some(3));
    /// assert_eq!(table.position(1), Some(4));
    /// assert_eq!(table.position(2), Some(5));
    /// assert_eq!(table.position(3), Some(8));
    /// assert_eq!(table.position(4), None);
    /// ```
    pub fn position(&self, pos: usize) -> Option<T> {
        // TODO: Change this to make it a reference
        if pos < self.sorted_vec.len() {
            let n = self.sorted_vec[pos].borrow().val.clone();
            Some(n)
        } else {
            None
        }
    }

    /// Get iterator over all elements sorted by number of occurrence (descending)
    ///
    /// # Examples
    ///
    /// ```
    /// use sqsh::stats::ProbTable;
    ///
    /// let mut table = ProbTable::<u8>::new();
    /// let data = vec![3, 4, 3, 3, 3, 3, 4, 5, 8];
    /// table.feed(&data);
    /// let mut iter = table.iter();
    ///
    /// assert_eq!(iter.next(), Some(3));
    /// assert_eq!(iter.next(), Some(4));
    /// assert_eq!(iter.next(), Some(5));
    /// assert_eq!(iter.next(), Some(8));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter(&self) -> TableIterator<T> {
        TableIterator {
            table: self,
            count: 0,
        }
    }

    /// Return capacity of table
    ///
    /// # Examples
    ///
    /// ```
    /// use sqsh::stats::ProbTable;
    ///
    /// let mut table = ProbTable::<u8>::new();
    /// let data = vec![3, 4, 3, 3, 3, 3, 4, 5, 8];
    /// table.feed(&data);
    ///
    /// assert_eq!(table.capacity(), 4);
    /// ```
    pub fn capacity(&self) -> usize {
        self.sorted_vec.capacity()
    }

    /// Return number of unique elements in table
    ///
    /// # Examples
    ///
    /// ```
    /// use sqsh::stats::ProbTable;
    ///
    /// let mut table = ProbTable::<u8>::new();
    /// let data = vec![3, 4, 3, 3, 3, 3, 4, 5, 8];
    /// table.feed(&data);
    ///
    /// assert_eq!(table.members(), 4);
    /// ```
    pub fn members(&self) -> usize {
        self.sorted_vec.len()
    }

    /// Get count of single element in table
    ///
    /// # Examples
    ///
    /// ```
    /// use sqsh::stats::ProbTable;
    ///
    /// let mut table = ProbTable::<u8>::new();
    /// let data = vec![3, 4, 3, 3, 3, 3, 4, 5, 8];
    /// table.feed(&data);
    ///
    /// assert_eq!(table.count(&3), Some(5));
    /// assert_eq!(table.count(&9), None);
    /// ```
    pub fn count(&self, val: &T) -> Option<usize> {
        match self.hm.get(val) {
            Some(node) => {
                let ix = node.borrow().count;
                Some(ix)
            }
            None => None,
        }
    }

    /// Renormalize sorted vector
    fn renormalize(&mut self, ix: usize) -> usize {
        let mut r = ix;
        // println!("Renormalizing starting ix '{:?}' in {:?}", ix, self);
        let ref_count = self.sorted_vec[ix].borrow().count;
        for (i, val) in self.sorted_vec.iter().enumerate().take(ix + 1) {
            if val.borrow().count < ref_count {
                r = i;
                break;
            }
        }
        self.swap(r, ix);
        r
    }

    /// Swap elements
    fn swap(&mut self, a: usize, b: usize) {
        self.sorted_vec[a].borrow_mut().rank = b;
        self.sorted_vec[b].borrow_mut().rank = a;
        self.sorted_vec.swap(a, b);
    }

    /// Check integrity of data structure
    fn is_coherent(&self) -> bool {
        if self.sorted_vec.is_empty() {
            return true;
        }
        let mut pred = self.sorted_vec.first().unwrap();
        let mut result = true;

        for (ix, node) in self.sorted_vec.iter().skip(1).enumerate() {
            // check if rank is ascending && if count is ascending
            if ix != pred.borrow().rank || pred.borrow().count < node.borrow().count {
                result = false;
                break;
            } else {
                pred = node;
            }
        }
        result
    }
}

pub struct TableIterator<'a, T>
where
    T: Countable + 'a,
{
    table: &'a ProbTable<T>,
    count: usize,
}

impl<'a, T> Iterator for TableIterator<'a, T>
where
    T: Countable + 'a,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < self.table.sorted_vec.len() {
            let result = self.table.position(self.count);
            self.count += 1;
            result
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_vec() {
        let mut test = ProbTable::new();
        test.insert(vec![3, 2, 1]);
        test.insert(vec![3, 2, 1]);
        test.insert(vec![3, 2, 1]);
        let mut iter = test.iter();

        assert_eq!(iter.next(), Some(vec![3, 2, 1]));
        assert_eq!(iter.next(), None);
        assert_eq!(test.sorted_vec.len(), 1);
        assert_eq!(test.hm.len(), 1);
        assert!(test.is_coherent());
    }

    #[test]
    fn test_insert_nodes() {
        let mut test = ProbTable::<u8>::new();
        test.insert(0);
        test.insert(1);
        test.insert(2);
        test.insert(3);
        assert_eq!(test.sorted_vec.len(), 4);
        assert_eq!(test.hm.len(), 4);
        assert!(test.is_coherent());
    }

    #[test]
    fn test_insert_same_nodes() {
        let mut test = ProbTable::<u8>::new();
        test.insert(0);
        test.insert(0);
        test.insert(1);
        test.insert(0);
        assert_eq!(test.sorted_vec.len(), 2);
        assert_eq!(test.hm.len(), 2);
        assert!(test.is_coherent());
    }

    #[test]
    fn test_insert_late_dominant() {
        let mut test = ProbTable::<u8>::new();
        test.insert(1);
        assert!(test.is_coherent());
        test.insert(0);
        test.insert(0);
        test.insert(0);
        assert!(test.is_coherent());
        test.insert(2);
        test.insert(2);
        test.insert(2);
        test.insert(2);
        assert_eq!(test.sorted_vec.len(), 3);
        assert_eq!(test.hm.len(), 3);
        assert!(test.is_coherent());
        assert_eq!(test.rank(&2), Some(0));
        assert_eq!(test.rank(&0), Some(1));
        assert_eq!(test.position(0), Some(2));
        let mut iter = test.iter();
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn edge_case() {
        let mut test = ProbTable::<u8>::new();
        let data = vec![3, 4, 3, 3, 4, 5, 5, 5, 7, 7, 7, 7];
        test.feed(&data);
        println!("{:?}", test);
        assert!(test.is_coherent())
    }
}
