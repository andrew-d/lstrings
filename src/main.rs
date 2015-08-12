extern crate ansi_term;
extern crate clap;
extern crate fern;
extern crate libc;
#[macro_use] extern crate log;
extern crate mmap;
extern crate time;

use clap::{Arg, App};

mod logger;
mod mm;


fn main() {
    let matches = App::new("lstrings")
        .version("0.0.1")
        .author("Andrew Dunham <andrew@du.nham.ca>")
        .about("Searches a file for strings, ranking by similarity to English")
        .arg(Arg::with_name("debug")
             .short("d")
             .multiple(true)
             .help("Sets the level of debugging information"))
        .arg(Arg::with_name("number")
             .short("n")
             .takes_value(true)
             .help("Specify the minimum string length"))
        // TODO: format, sort options, etc.
        .arg(Arg::with_name("input")
             .help("Sets the input file(s) to search")
             .required(true)
             .multiple(true)
             )
        .get_matches();

    logger::init_logger_config(&matches);

    let number = {
        let arg = matches.value_of("number").unwrap_or("4");

        match usize::from_str_radix(arg, 10) {
            Ok(n) => n,
            Err(_) => {
                error!("Invalid argument for 'number': {}", arg);
                return;
            },
        }
    };

    let input_paths = matches.values_of("input").unwrap();

    for path in input_paths {
        info!("Searching file: {}", path);
        search_file(path, number);
    }
}

// Search the given input file for all strings and print them.
fn search_file<P>(path: P, min_len: usize)
where P: std::convert::AsRef<std::path::Path>
{
    let path = path.as_ref();

    mm::with_file_mmap(path, |map| {
        let mut result = vec![];

        for ch in map {
            if is_printable(*ch) {
                result.push(*ch);
                continue;
            }

            if result.len() > min_len {
                let s = std::str::from_utf8(&*result).unwrap();
                println!("{}", s);
            }

            result.clear();
        }
    });
}

#[inline(always)]
fn is_printable(ch: u8) -> bool {
    ch > 0x1F && ch < 0x7F
}
