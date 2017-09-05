use std::fmt;
use std::cmp::{self, Ordering};
use std::collections::BTreeSet;
use std::collections::btree_set::Iter;

#[derive(Debug, Clone, Default)]
pub struct FreeRanges {
    free_list: BTreeSet<Range>,
}

impl FreeRanges {
    /// Starts empty with no ranges free
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Initializes FreeRanges with 0...usize::MAX already free
    #[inline]
    pub fn with_all_free() -> FreeRanges {
        FreeRanges::with_initial_range(Range {
            min: 0,
            max: std::usize::MAX,
        })
    }

    /// Initializes FreeRanges with the passed `range` already marked as free
    #[inline]
    pub fn with_initial_range(range: Range) -> FreeRanges {
        let mut ranges = FreeRanges::new();
        ranges.free_list.insert(range);
        ranges
    }

    /// Iterator over all of the contiguous free ranges
    #[inline]
    pub fn free_ranges(&self) -> Iter<Range> {
        self.free_list.iter()
    }

    /// Marks a specific index as free
    #[inline]
    pub fn set_free(&mut self, index: usize) {
        if self.free_list.contains(&Range::id(index)) {
            return;
        }

        let range = Range::id(index);

        let range_front = if index > 0 { range.push_front() } else { range };
        let range_back = range.push_back();
        let combine_front = self.free_list.get(&range_front).cloned();
        let combine_back = self.free_list.get(&range_back).cloned();

        match (combine_front, combine_back) {
            (Some(front_range), Some(back_range)) => {
                let combined = front_range.merge(range).merge(back_range);

                self.free_list.remove(&front_range);
                self.free_list.remove(&back_range);
                self.free_list.insert(combined);
            }
            (Some(front_range), None) => {
                let combined = front_range.merge(range);

                self.free_list.remove(&front_range);
                self.free_list.insert(combined);
            }
            (None, Some(back_range)) => {
                let combined = back_range.merge(range);

                self.free_list.remove(&back_range);
                self.free_list.insert(combined);
            }
            (None, None) => {
                self.free_list.insert(range);
            }
        }
    }

    /// Marks a free index as used. Returns false if the index was not free
    #[inline]
    pub fn set_used(&mut self, index: usize) -> bool {
        let range = Range::id(index);

        if let Some(&intersecting) = self.free_list.get(&range) {
            self.free_list.remove(&intersecting);
            let (left, right) = intersecting.split(index);
            if !left.empty() {
                self.free_list.insert(left);
            }
            if !right.empty() {
                self.free_list.insert(right);
            }
            true
        } else {
            false
        }
    }

    /// Returns the first free value if one exists
    #[inline]
    pub fn first(&self) -> Option<usize> {
        self.free_list.iter().nth(0).map(|r| r.min)
    }

    /// Marks the first index in the free list as used and returns it
    #[inline]
    pub fn set_first_used(&mut self) -> Option<usize> {
        if let Some(&first) = self.free_list.iter().nth(0) {
            self.free_list.remove(&first);
            let range = first.pop_front();
            if !range.empty() {
                self.free_list.insert(range);
            }
            return Some(first.min)
        }

        None
    }

    /// Returns the first free value if one exists
    #[inline]
    pub fn last(&self) -> Option<usize> {
        self.free_list.iter().rev().nth(0).map(|r| r.max)
    }

    /// Marks the first index in the free list as used and returns it
    #[inline]
    pub fn set_last_used(&mut self) -> Option<usize> {
        if let Some(&last) = self.free_list.iter().rev().nth(0) {
            self.free_list.remove(&last);
            if last.max != 0 {
                let range = last.pop_back();
                if !range.empty() {
                    self.free_list.insert(range);
                }
            }
            return Some(last.max);
        }

        None
    }

    #[inline]
    pub fn remove_last_contiguous(&mut self) {
        if let Some(last) = self.last() {
            self.free_list.remove(&Range::id(last));
        }
    }
}

#[derive(Copy, Clone)]
pub struct Range {
    pub min: usize,
    pub max: usize,
}

impl fmt::Debug for Range {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "({}...{})", self.min, self.max)
    }
}

impl Range {
    #[inline]
    pub fn id(id: usize) -> Self {
        Range { min: id, max: id }
    }

    #[inline]
    pub fn empty(self) -> bool {
        self.min >= self.max
    }

    #[inline]
    pub fn push_front(mut self) -> Self {
        self.min -= 1;
        self
    }

    #[inline]
    pub fn push_back(mut self) -> Self {
        self.max += 1;
        self
    }

    #[inline]
    pub fn pop_front(mut self) -> Self {
        self.min += 1;
        self
    }

    #[inline]
    pub fn pop_back(mut self) -> Self {
        self.max -= 1;
        self
    }

    #[inline]
    pub fn merge(self, other: Self) -> Self {
        Range {
            min: cmp::min(self.min, other.min),
            max: cmp::max(self.max, other.max),
        }
    }

    #[inline]
    pub fn contains(&self, value: usize) -> bool {
        value >= self.min && value <= self.max
    }

    #[inline]
    pub fn split(self, middle: usize) -> (Range, Range) {
        let left = Range {
            min: self.min,
            max: middle - 1,
        };
        let right = Range {
            min: middle + 1,
            max: self.max,
        };
        (left, right)
    }
}

impl PartialEq for Range {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Range {}

impl PartialOrd for Range {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Range {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        if self.contains(other.min) || self.contains(other.max) {
            return Ordering::Equal;
        }

        self.min.cmp(&other.min)
    }
}
