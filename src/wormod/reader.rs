use super::file;
use super::memory;
use crate::params::Params;
use crate::print::*;

use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::str::from_utf8;

pub(super) type Reader = BufReader<Box<dyn std::io::Read>>;

pub(super) fn buffered_reader(params: &Params) -> (Reader, usize) {
    let available_memory = memory::available_memory();
    let buffer_size = memory::buffer_size(available_memory);
    let buf_reader : Reader;
    let file_size : usize;

    if let Some(in_path) = params.input.as_ref() {
        params.check_input_path();
        let in_file = file::open_input_file(in_path);
        file_size = file::file_size(&in_file, in_path);
        if file_size == 0 {
            exit_err!(
                ("The input file is empty"),
                ("This is equivalent to a no-op")
            );
        } else if params.sort || params.unique {
            // the whole file must be stored in-memory
            if file_size >= available_memory - buffer_size * 5 {
                exit_err!(
                    ("Available memory is too low"),
                    ("Not enough memory to perform the requested operation(s)")
                );
            }
        }
        buf_reader = BufReader::with_capacity(buffer_size, Box::new(in_file));
    } else {
        // reading from standard input
        file_size = 0;
        buf_reader = BufReader::with_capacity(buffer_size, Box::new(std::io::stdin()));
    }

    (buf_reader, file_size)
}

pub(super) fn read_from_file(mut reader: Reader, file_size: usize) -> String {
    let available_memory = memory::available_memory();
    if !memory::is_memory_enough_with(available_memory, file_size) {
        exit_err!(
            ("Not enough memory to read the input file")
        );
    }
    let mut buffer = String::with_capacity(file_size);
    if let Err(e) = reader.read_to_string(&mut buffer) {
        exit_err!(
            ("Failed to read input file: {}", e.to_string())
        );
    }
    buffer
}

pub(super) fn read_from_stdin(mut buf_reader: Reader) -> String {
    let check_memory = || {
        if !memory::enough_memory_left() {
            exit_err!(
                ("Not enough memory to keep reading")
            );
        }
    };
    let mut buffer = String::new();
    check_memory();
    let mut read_buf = vec![0; memory::IO_BUF_SIZE];
    loop {
        check_memory();
        match buf_reader.read(read_buf.as_mut_slice()) {
            Err(e) => {
                match e.kind() {
                    std::io::ErrorKind::Interrupted => continue,
                    _ => {
                        exit_err!(
                            ("Failed to read: {}", e.to_string())
                        );
                    }
                }
            },
            Ok(0) => break, // reached EOF
            Ok(n) => {
                debug_assert!(n <= memory::IO_BUF_SIZE);
                match from_utf8(&read_buf[..n]) {
                    Ok(slice) => buffer.push_str(slice),
                    Err(e) => {
                        exit_err!(
                            ("Non-UTF8 character found: {}", e.to_string())
                        );
                    },
                }
            },
        }
    }
    buffer
}

pub(super) fn pipe_read(reader: &mut Reader, buffer: &mut String) {
    if let Err(e) = reader.read_line(buffer) {
        exit_err!(
            ("Failed to read: {}", e.to_string())
        );
    }
}
