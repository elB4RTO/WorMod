use crate::print::*;

use sysinfo::System;

/// The size of 1 MiB
#[allow(non_upper_case_globals)]
pub(super) const MiB : usize = 1_048_576;

/// The default size for the I/O buffers
pub(super) const IO_BUF_SIZE : usize = 16 * MiB;

/// The minimum amount of memory that must be available
/// in order to not risk to harm the system
const MIN_AVL_MEM : usize = 64 * MiB;

/// Retrieves the available memory left on the system
pub(super) fn available_memory() -> usize {
    let mut sys = System::new();
    sys.refresh_memory();
    sys.available_memory() as usize
}

pub(super) fn enough_memory_left() -> bool {
    available_memory() >= MIN_AVL_MEM
}

pub(super) fn is_memory_enough_with(avl_mem: usize, take_mem: usize) -> bool {
    avl_mem - take_mem >= MIN_AVL_MEM
}

/// Returns the size for the I/O buffers
///
/// Calls terminate with a failure code if the available memory
/// left on the system is too low
pub(super) fn buffer_size(avl_mem: usize) -> usize {
    if avl_mem < MIN_AVL_MEM {
        fail_low_memory(avl_mem);
    }
    IO_BUF_SIZE
}

/// Terminates the process with a failure code
fn fail_low_memory(avl_mem: usize) -> ! {
    let avl_mib = avl_mem as f64 / 1048576.0;
    exit_err!(
        ("Available memory is too low: {:.4} MiB", avl_mib)
    );
}
