/*
 * This file will create a bigram map from /usr/share/dict/words and write it
 * to an output file.
 */

#![feature(vec_resize)]

extern crate bincode;
extern crate rustc_serialize;

use std::env::args;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

use bincode::{SizeLimit};
use bincode::rustc_serialize::encode;

#[path = "../src/bigram.rs"]
mod bigram;


fn main() {
    let args = args().collect::<Vec<String>>();
    let mut bg = bigram::BigramMap::new();

    let f = File::open(&args[1]).unwrap();
    let file = BufReader::new(&f);

    for (i, line) in file.lines().enumerate() {
        if i % 1000 == 0 {
            println!("processing line {}", i);
        }

        let l = line.unwrap();
        bg.add(l);
    }

    println!("finished processing, encoding...");

    let encoded: Vec<u8> = encode(&bg, SizeLimit::Infinite).unwrap();
    println!("finished encoding ({} bytes)", encoded.len());

    let mut out = File::create(&args[2]).unwrap();
    out.write_all(&*encoded).unwrap();

    println!("done!");
}
