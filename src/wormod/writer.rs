use super::file;
use super::memory;
use crate::params::Params;
use crate::print::*;

use std::io::BufWriter;
use std::io::Write;

pub(super) type Writer = BufWriter<Box<dyn std::io::Write>>;

/// Line feed
const LF : [u8;1] = [0xA];

pub(super) fn buffered_writer(params: &Params) -> Writer {
    let available_memory = memory::available_memory();
    let buffer_size = memory::buffer_size(available_memory);
    let buf_writer : Writer;

    if let Some(out_path) = params.output.as_ref() {
        params.check_output_path();
        let out_file = file::open_output_file(out_path, params.append_output);
        buf_writer = BufWriter::with_capacity(buffer_size, Box::new(out_file));
    } else {
        // writing to standard output
        buf_writer = BufWriter::with_capacity(buffer_size, Box::new(std::io::stdout()));
    }

    buf_writer
}

pub(super) fn write_to_file(mut writer: Writer, wordlist: Vec<String>) {
    for buf in wordlist.iter().filter(|s| !s.is_empty()) {
        if let Err(e) = writer.write_all(buf.as_bytes()) {
            exit_err!(
                ("Failed to entirely write output file: {}", e.to_string())
            );
        }
        if let Err(e) = writer.write(&LF) {
            match e.kind() {
                std::io::ErrorKind::Interrupted => continue,
                _ => {
                    exit_err!(
                        ("Failed to write: {}", e.to_string())
                    );
                }
            }
        }
    }
}

pub(super) fn write_to_stdout(mut writer: Writer, wordlist: Vec<String>) {
    for buf in wordlist.iter().filter(|s| !s.is_empty()) {
        if let Err(e) = writer.write_all(buf.as_bytes()) {
            exit_err!(
                ("Failed to entirely write to standard output: {}", e.to_string())
            );
        }
        if let Err(e) = writer.write(&LF) {
            match e.kind() {
                std::io::ErrorKind::Interrupted => continue,
                _ => {
                    exit_err!(
                        ("Failed to write: {}", e.to_string())
                    );
                }
            }
        }
    }
}

pub(super) fn pipe_write(writer: &mut Writer, buffer: &String) {
    if let Err(e) = writer.write_all(buffer.as_bytes()) {
        exit_err!(
            ("Failed to write: {}", e.to_string())
        );
    } else if let Err(e) = writer.write_all(&LF) {
        exit_err!(
            ("Failed to write: {}", e.to_string())
        );
    } else if let Err(e) = writer.flush() {
        exit_err!(
            ("Failed to write: {}", e.to_string())
        );
    }
}
