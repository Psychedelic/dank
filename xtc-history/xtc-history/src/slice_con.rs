pub struct SliceCon<'a, T> {
    parts: Vec<Part<'a, T>>,
    virtual_gap_index: usize,
    index: Vec<u64>,
}

enum Part<'a, T> {
    Slice(&'a [T]),
    Virtual { len: usize, index: usize },
}

#[derive(PartialOrd, PartialEq, Debug)]
pub enum GetResult<'a, T> {
    Data(&'a T),
    VirtualGap { id: usize, index: usize },
    OutOfRange,
}

impl<'a, T> SliceCon<'a, T> {
    #[inline]
    pub fn add_virtual_gap(&mut self, len: usize) -> usize {
        let index = self.virtual_gap_index;
        self.virtual_gap_index += 1;
        if len == 0 {
            return index;
        }
        self.parts.push(Part::Virtual { len, index });
        self.index
            .push(self.index.last().cloned().unwrap_or(0) + len as u64);
        index
    }

    #[inline]
    pub fn add_slice(&mut self, slice: &'a [T]) {
        let len = slice.len();
        if len == 0 {
            return;
        }
        self.parts.push(Part::Slice(slice));
        self.index
            .push(self.index.last().cloned().unwrap_or(0) + len as u64);
    }

    #[inline]
    pub fn get(&self, index: u64) -> GetResult<'a, T> {
        let part_index = match self.index.binary_search(&index) {
            Ok(i) => i + 1,
            Err(i) => i,
        };

        let inner_index = match part_index {
            i if i == self.parts.len() => return GetResult::OutOfRange,
            0 => index,
            i => index - self.index[i - 1],
        } as usize;

        match &self.parts[part_index] {
            Part::Slice(s) => GetResult::Data(&s[inner_index]),
            Part::Virtual { index, .. } => GetResult::VirtualGap {
                id: *index,
                index: inner_index,
            },
        }
    }
}

impl<'a, T> Default for SliceCon<'a, T> {
    fn default() -> Self {
        SliceCon {
            parts: Vec::with_capacity(8),
            virtual_gap_index: 0,
            index: Vec::with_capacity(0),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_slice_virtual_gaps() {
        // 0 1 2          3
        // 3 4 5 6 7      3  8
        //                3  8  8
        // 8 9            3  8  8  10
        let mut v = SliceCon::<()>::default();
        v.add_virtual_gap(3);
        v.add_virtual_gap(5);
        v.add_virtual_gap(0);
        v.add_virtual_gap(0);
        v.add_virtual_gap(0);
        v.add_virtual_gap(2);
        assert_eq!(v.get(0), GetResult::VirtualGap { id: 0, index: 0 });
        assert_eq!(v.get(1), GetResult::VirtualGap { id: 0, index: 1 });
        assert_eq!(v.get(2), GetResult::VirtualGap { id: 0, index: 2 });
        assert_eq!(v.get(3), GetResult::VirtualGap { id: 1, index: 0 });
        assert_eq!(v.get(4), GetResult::VirtualGap { id: 1, index: 1 });
        assert_eq!(v.get(5), GetResult::VirtualGap { id: 1, index: 2 });
        assert_eq!(v.get(6), GetResult::VirtualGap { id: 1, index: 3 });
        assert_eq!(v.get(7), GetResult::VirtualGap { id: 1, index: 4 });
        assert_eq!(v.get(8), GetResult::VirtualGap { id: 5, index: 0 });
        assert_eq!(v.get(9), GetResult::VirtualGap { id: 5, index: 1 });
        assert_eq!(v.get(10), GetResult::OutOfRange);
    }

    #[test]
    fn test_slice_data() {
        let mut v = SliceCon::<u64>::default();
        v.add_slice(&[0, 1, 2, 3]);
        v.add_slice(&[4, 5]);
        v.add_slice(&[]);
        v.add_slice(&[6]);
        v.add_slice(&[7]);
        for i in 0..=7u64 {
            assert_eq!(v.get(i), GetResult::Data(&i));
        }
        assert_eq!(v.get(8), GetResult::OutOfRange);
    }
}
