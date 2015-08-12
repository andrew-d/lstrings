extern crate ansi_term;
extern crate clap;
extern crate fern;
extern crate itertools;
extern crate libc;
#[macro_use] extern crate log;
extern crate mmap;
extern crate time;

use std::cmp;

use clap::{Arg, App};
use itertools::Itertools;

mod logger;
mod mm;


#[derive(Debug, Clone, Copy)]
enum FormatOption {
    NoFormat,
    Decimal,
    Octal,
    Hexadecimal,
}


impl FormatOption {
    fn from_str(s: &str) -> FormatOption {
        match s {
            ""  => FormatOption::NoFormat,
            "d" => FormatOption::Decimal,
            "o" => FormatOption::Octal,
            "x" => FormatOption::Hexadecimal,
            _   => unreachable!(),
        }
    }
}


#[derive(Debug, Clone, Copy)]
enum SortOption {
    /// Sort by start address
    Address,

    /// Sort by length of the string.
    Length,

    /// Sort by likelyhood that the string is English text.
    English,
}


impl SortOption {
    fn from_str(s: &str) -> SortOption {
        match s {
            ""        => SortOption::Address,
            "address" => SortOption::Address,
            "length"  => SortOption::Length,
            "english" => SortOption::English,
            _         => unreachable!(),
        }
    }
}


#[derive(Debug, Clone, Copy)]
enum SortDirection {
    Ascending,
    Descending,
}


fn main() {
    let format_choices = ["", "d", "o", "x"];
    let sort_choices = ["", "address", "length", "english"];

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
        .arg(Arg::with_name("format")
             .short("t")
             .takes_value(true)
             .possible_values(&format_choices)
             .help("If given, specify the output format for each string"))
        .arg(Arg::with_name("sort")
             .short("s")
             .takes_value(true)
             .possible_values(&sort_choices)
             .help("Specify how to sort the found strings"))
        .arg(Arg::with_name("reverse")
             .short("r")
             .help("Reverse the sort order (i.e. descending order)"))
        .arg(Arg::with_name("input")
             .help("Sets the input file(s) to search")
             .required(true)
             .multiple(true))
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
    let format = FormatOption::from_str(matches.value_of("format").unwrap_or(""));
    let sort = SortOption::from_str(matches.value_of("sort").unwrap_or("address"));
    let direction = if matches.is_present("reverse") {
        SortDirection::Descending
    } else {
        SortDirection::Ascending
    };
    let input_paths = matches.values_of("input").unwrap();

    for path in input_paths {
        info!("Searching file: {}", path);
        search_file(path, number, format, sort, direction);
    }
}


#[derive(Debug, Clone, Copy)]
struct FoundString(usize, usize);

impl FoundString {
    fn start(&self) -> usize {
        let FoundString(start, _) = *self;
        start
    }

    fn end(&self) -> usize {
        let FoundString(_, end) = *self;
        end
    }

    fn len(&self) -> usize {
        self.end() - self.start()
    }

    fn slice<'a>(&self, arr: &'a [u8]) -> &'a [u8] {
        let FoundString(start, end) = *self;
        &arr[start..end]
    }

    fn as_str<'a>(&self, arr: &'a [u8]) -> &'a str {
        let FoundString(start, end) = *self;
        let bytes = &arr[start..end];

        std::str::from_utf8(bytes).unwrap()
    }
}

// Search the given input file for all strings and print them.
fn search_file<P>(path: P, min_len: usize, format: FormatOption, sort: SortOption, dir: SortDirection)
where P: std::convert::AsRef<std::path::Path>
{
    let path = path.as_ref();

    mm::with_file_mmap(path, |map| {
        let mut results = vec![];

        let mut start = None;

        for (i, ch) in map.iter().enumerate() {
            if is_printable(*ch) {
                if start.is_none() {
                    start = Some(i)
                }

                continue;
            }

            // Not printable.  If we started a printable string...
            if let Some(starti) = start {
                // ... and the string is within the length requirement ...
                if (i - starti) >= min_len {
                    // ... save the result.
                    results.push(FoundString(starti, i));
                }
            }

            // Reset the start.
            start = None;
        }

        // Sort the results.
        let sort_func: fn(&FoundString, &FoundString) -> cmp::Ordering = match (sort, dir) {
            (SortOption::Address, SortDirection::Ascending)  => sort_address_asc,
            (SortOption::Address, SortDirection::Descending) => sort_address_desc,
            (SortOption::Length, SortDirection::Ascending)   => sort_length_asc,
            (SortOption::Length, SortDirection::Descending)  => sort_length_desc,
            (SortOption::English, SortDirection::Ascending)  => panic!("unsupported"),
            (SortOption::English, SortDirection::Descending) => panic!("unsupported"),
        };

        let sorted_results = results
            .into_iter()
            .sort_by(sort_func);

        // Print all results.
        for res in sorted_results {
            let prefix = match format {
                FormatOption::Decimal     => format!("{} ", res.start()),
                FormatOption::Octal       => format!("{:o} ", res.start()),
                FormatOption::Hexadecimal => format!("{:x} ", res.start()),
                FormatOption::NoFormat    => String::new(),
            };

            println!("{}{}", prefix, res.as_str(map));
        }
    });
}

#[inline(always)]
fn is_printable(ch: u8) -> bool {
    ch > 0x1F && ch < 0x7F
}

#[inline(always)]
fn sort_address_asc(a: &FoundString, b: &FoundString) -> cmp::Ordering {
    Ord::cmp(&a.start(), &b.start())
}

#[inline(always)]
fn sort_address_desc(a: &FoundString, b: &FoundString) -> cmp::Ordering {
    Ord::cmp(&b.start(), &a.start())
}

#[inline(always)]
fn sort_length_asc(a: &FoundString, b: &FoundString) -> cmp::Ordering {
    Ord::cmp(&a.len(), &b.len())
}

#[inline(always)]
fn sort_length_desc(a: &FoundString, b: &FoundString) -> cmp::Ordering {
    Ord::cmp(&b.len(), &a.len())
}
