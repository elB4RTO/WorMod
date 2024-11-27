use super::memory;
use crate::print::print_err;

pub(super) trait FromBuffer {
    fn from_buffer(buffer: String) -> Self;
}

impl FromBuffer for Vec<String> {
    fn from_buffer(buffer: String) -> Self {
        let entries = buffer.trim().split('\n').filter(|e| !e.is_empty());
        let n_entries = entries.clone().count();
        {
            let content_size = buffer.len() - n_entries;
            let collection_size = n_entries * std::mem::size_of::<String>();
            let wbuf_size = collection_size + content_size;
            let available_memory = memory::available_memory();
            if !memory::is_memory_enough_with(available_memory, wbuf_size) {
                print_err!("Not enough memory to complete the operation(s)");
                std::process::exit(1);
            }
        }
        entries.map(|e| e.to_owned()).collect()
    }
}


pub(super) trait DedupUnsorted {
    fn dedup_unsorted(&mut self);
}

impl DedupUnsorted for Vec<String> {
    fn dedup_unsorted(&mut self) {
        let len = self.len();
        let mut max = len;
        let mut i = 0;
        while i < max {
            let mut j = i + 1;
            while j < max {
                if self[j] == self[i] {
                    let mut t = j + 1;
                    while t < max && self[t] == self[i] {
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
