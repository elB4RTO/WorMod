use crate::print::print_err;

use std::fs::File;
use std::fs::OpenOptions;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

pub(super) fn open_input_file(path: &PathBuf) -> File {
    OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|e| {
            print_err!("Failed to open input file: {:?}", path);
            print_err!("Reason of the failure: {}", e.to_string());
            std::process::exit(1);
        }).unwrap()
}

pub(super) fn open_output_file(path: &PathBuf, append_mode: bool) -> File {
    OpenOptions::new()
        .create(true)
        .write(true)
        .append(append_mode)
        .truncate(!append_mode)
        .open(path)
        .map_err(|e| {
            print_err!("Failed to open output file: {:?}", path);
            print_err!("Reason of the failure: {}", e.to_string());
            std::process::exit(1);
        }).unwrap()
}

pub(super) fn file_size(file: &File, path: &PathBuf) -> usize {
    file.metadata()
        .map_err(|e| {
            print_err!("Failed to retrieve file size: {:?}", path);
            print_err!("Reason of the failure: {}", e.to_string());
            std::process::exit(1);
        }).unwrap()
        .size() as usize
}
