mod file;
mod memory;
mod reader;
mod wordlist;
mod writer;

use reader::Reader;
use wordlist::*;
use writer::Writer;
use crate::params::Params;
use crate::print::*;

use unicode_segmentation::UnicodeSegmentation;

type RunResult = Result<(),Box<dyn std::error::Error>>;

pub(crate) fn run(params: Params) -> RunResult {
    let (buf_reader, file_size) = reader::buffered_reader(&params);
    let buf_writer = writer::buffered_writer(&params);

    if params.pipe {
        pipe_mode(params, buf_reader, buf_writer);
    } else {
        stock_mode(params, buf_reader, buf_writer, file_size);
    }

    Ok(())
}

fn stock_mode(
    params: Params,
    buf_reader: Reader,
    buf_writer: Writer,
    file_size: usize,
) {
    let buffer = if params.input.is_some() {
        reader::read_from_file(buf_reader, file_size)
    } else {
        reader::read_from_stdin(buf_reader)
    };

    let mut wordlist = Vec::from_buffer(buffer);

    if params.has_length_range() {
        let min_len = params.min_len.unwrap_or(0);
        let max_len = params.max_len.unwrap_or(usize::MAX);
        wordlist.retain(|s| {
            let entry_len = s.len();
            (min_len <= entry_len) & (entry_len <= max_len)
        })
    }

    if params.reverse {
        wordlist.iter_mut().for_each(|e| {
            *e = e.graphemes(true).rev().collect::<String>()
        });
    }

    if params.sort && params.unique {
        wordlist.sort_unstable();
        wordlist.dedup();
    } else if params.sort {
        wordlist.sort_unstable();
    } else if params.unique {
        wordlist.dedup_unsorted();
    }

    if params.output.is_some() {
        writer::write_to_file(buf_writer, wordlist);
    } else {
        writer::write_to_stdout(buf_writer, wordlist);
    }
}

fn pipe_mode(
    params: Params,
    ref mut buf_reader: Reader,
    ref mut buf_writer: Writer,
) {
    let min_len = params.min_len.unwrap_or(0);
    let max_len = params.max_len.unwrap_or(usize::MAX);

    let ref mut buffer = String::with_capacity(memory::MiB);
    let mut unique_entries = Vec::new();
    loop {
        reader::pipe_read(buf_reader, buffer);

        *buffer = buffer.trim().to_owned();
        if buffer.is_empty() {
            // reached EOF
            break;
        }

        let entry_len = buffer.len();
        if (entry_len < min_len) | (max_len < entry_len) {
            buffer.clear();
            continue;
        }

        if params.unique {
            if unique_entries.contains(buffer) {
                buffer.clear();
                continue;
            }
            let buffer_size = buffer.len();
            let available_memory = memory::available_memory();
            if !memory::is_memory_enough_with(available_memory, buffer_size) {
                exit_err!(
                    ("Not enough memory to complete the operation(s)")
                );
            }
            unique_entries.push(buffer.clone());
        }

        if params.reverse {
            *buffer = buffer.graphemes(true).rev().collect::<String>();
        }

        writer::pipe_write(buf_writer, buffer);

        buffer.clear();
    }
}
