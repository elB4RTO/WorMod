
pub(super) trait DedupUnsorted {
    fn dedup_unsorted(&mut self);
}

impl DedupUnsorted for Vec<&str> {
    fn dedup_unsorted(&mut self) {
        let len = self.len();
        let mut max = len;
        let mut i = 0;
        while i < max {
            let entry = self[i];
            let mut j = i + 1;
            while j < max {
                if self[j] == entry {
                    let mut t = j + 1;
                    while t < max && self[t] == entry {
                        t += 1;
                    }
                    let n_shifts = t - j;
                    self[j..].rotate_left(n_shifts);
                    max -= n_shifts;
                    continue;
                }
                j += 1;
            }
            i += 1;
        }
        self.truncate(max);
    }
}
