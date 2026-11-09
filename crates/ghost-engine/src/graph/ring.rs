pub struct RingBuffer<T> {
    data: Box<[T]>,
    head: usize,
    len: usize,
}

impl<T: Copy + Default> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "ring buffer capacity must be positive");
        Self {
            data: vec![T::default(); capacity].into_boxed_slice(),
            head: 0,
            len: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.data.len()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn readable(&self) -> &[T] {
        let end = (self.head + self.len).min(self.data.len());
        &self.data[self.head..end]
    }

    pub fn writable(&mut self) -> &mut [T] {
        let capacity = self.data.len();
        let free = capacity - self.len;
        let tail = (self.head + self.len) % capacity;
        let end = (tail + free).min(capacity);
        &mut self.data[tail..end]
    }

    pub fn consume(&mut self, count: usize) {
        assert!(count <= self.len, "consumed more than available");
        self.len -= count;
        if self.len == 0 {
            self.head = 0;
        } else {
            self.head = (self.head + count) % self.data.len();
        }
    }

    pub fn produce(&mut self, count: usize) {
        assert!(
            count <= self.data.len() - self.len,
            "produced more than free space"
        );
        self.len += count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push(ring: &mut RingBuffer<u32>, values: &[u32]) -> usize {
        let w = ring.writable();
        let n = w.len().min(values.len());
        w[..n].copy_from_slice(&values[..n]);
        ring.produce(n);
        n
    }

    #[test]
    fn starts_empty_with_full_writable_space() {
        let mut ring = RingBuffer::<u32>::new(4);
        assert!(ring.is_empty());
        assert_eq!(ring.capacity(), 4);
        assert!(ring.readable().is_empty());
        assert_eq!(ring.writable().len(), 4);
    }

    #[test]
    fn full_buffer_has_no_writable_space() {
        let mut ring = RingBuffer::<u32>::new(4);
        assert_eq!(push(&mut ring, &[1, 2, 3, 4]), 4);
        assert_eq!(ring.len(), 4);
        assert!(ring.writable().is_empty());
        assert_eq!(ring.readable(), &[1, 2, 3, 4]);
    }

    #[test]
    fn wraps_around_with_contiguous_views() {
        let mut ring = RingBuffer::<u32>::new(4);
        push(&mut ring, &[1, 2, 3]);
        ring.consume(2);
        assert_eq!(ring.readable(), &[3]);
        assert_eq!(push(&mut ring, &[4, 5, 6]), 1);
        assert_eq!(ring.readable(), &[3, 4]);
        assert_eq!(push(&mut ring, &[5, 6]), 2);
        assert_eq!(ring.len(), 4);
        assert_eq!(ring.readable(), &[3, 4]);
        ring.consume(2);
        assert_eq!(ring.readable(), &[5, 6]);
        ring.consume(2);
        assert!(ring.is_empty());
        assert_eq!(ring.writable().len(), 4);
    }

    #[test]
    fn preserves_order_over_many_cycles() {
        let mut ring = RingBuffer::<u32>::new(5);
        let mut next_in = 0u32;
        let mut next_out = 0u32;
        for round in 0..200 {
            let want = (round % 4) + 1;
            for _ in 0..want {
                if push(&mut ring, &[next_in]) == 1 {
                    next_in += 1;
                }
            }
            let take = ring.readable().len().min(3);
            for &v in &ring.readable()[..take] {
                assert_eq!(v, next_out);
                next_out += 1;
            }
            ring.consume(take);
        }
        assert!(next_out > 100);
    }

    #[test]
    #[should_panic(expected = "consumed more than available")]
    fn over_consume_panics() {
        let mut ring = RingBuffer::<u32>::new(2);
        ring.consume(1);
    }

    #[test]
    #[should_panic(expected = "produced more than free space")]
    fn over_produce_panics() {
        let mut ring = RingBuffer::<u32>::new(2);
        ring.produce(3);
    }
}
