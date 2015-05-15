//
// MSX CAS Packager
// Copyright (c) 2015 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate docopt;
extern crate rustc_serialize;

mod args;
mod tape;

use std::fs::File;
use std::io::Write;

#[allow(dead_code)]
fn main() {
    let cmd = args::parse();
    match cmd {
        args::Command::Version => print_version(),
        args::Command::List(path) => list_files(&path[..]),
        args::Command::Extract(path) => extract_all(&path[..]),
    };
}

fn print_version() {
    println!("MSX CAS Packager (MCP) v0.1.0");
    println!("Copyright (C) 2015 Alvaro Polo");
    println!("");
    println!("This program is subject to the terms of the Mozilla Public License v2.0.");
    println!("");
}

macro_rules! open_tape {
    ($path: expr) => ({
        let mut tape_file = match File::open($path) {
            Ok(f) => f,
            Err(e) => {
                println!("Cannot open file '{}': {}", $path, e);
                return
            }
        };
        match tape::Tape::read(&mut tape_file) {
            Ok(f) => f,
            Err(e) => {
                println!("Cannot read file '{}': {}", $path, e);
                return
            }
        }
    })
}

macro_rules! create_file {
    ($path: expr) => ({
        match File::create($path) {
            Ok(f) => f,
            Err(e) => {
                println!("Cannot create file '{}': {}", $path, e);
                return
            }
        }
    })
}

macro_rules! write_file {
    ($name: expr, $file: expr, $data: expr) => ({
        match $file.write_all($data) {
            Ok(_) => {},
            Err(e) => {
                println!("Cannot write to file '{}': {}", $name, e);
            },
        };
    })
}

fn list_files(path: &str) {
    let tape = open_tape!(path);
    for file in tape.files() {
        match file {
            tape::File::Bin(name, begin, end, start, data) => {
                println!("bin    | {:6} | {:5} bytes | [0x{:x},0x{:x}]:0x{:x}",
                    name, data.len(), begin, end, start);
            },
            tape::File::Basic(name, data) => {
                println!("basic  | {:6} | {:5} bytes |", name, data.len());
            },
            tape::File::Ascii(name, data) => {
                let nbytes = data.iter().fold(0, |size, chunk| size + chunk.len());
                println!("ascii  | {:6} | {:5} bytes |", name, nbytes);
            },
            tape::File::Custom(data) => {
                println!("custom |        | {:5} bytes |", data.len());
            }
        };
    }
}

fn extract_all(path: &str) {
    let tape = open_tape!(path);
    let mut next_custom = 0;
    for file in tape.files() {
        let out_path = file.name()
            .map(|n| n.to_string())
            .unwrap_or_else(|| format!("custom.{:03}", { next_custom += 1; next_custom }));
        print!("Extracting {}... ", out_path);
        extract_file(&file, &out_path[..]);
        println!("Done");
    }
}

fn extract_file(file: &tape::File, out_path: &str) {
    let mut ofile = create_file!(out_path);
    match file {
        &tape::File::Bin(_, _, _, _, data) => {
            write_file!(out_path, ofile, data);
        },
        &tape::File::Basic(_, data) => {
            write_file!(out_path, ofile, data);
        },
        &tape::File::Ascii(_, ref chunks) => {
            for chunk in chunks {
                write_file!(out_path, ofile, chunk);
            }
        },
        &tape::File::Custom(ref data) => {
            write_file!(out_path, ofile, data);
        },
    }
}
