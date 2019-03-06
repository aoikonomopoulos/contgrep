extern crate clap;
extern crate regex;

use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::process::exit;

use clap::{App, Arg};
use regex::bytes::{Regex, RegexSet, RegexSetBuilder};

enum State {
    ReadingLine,
    ReadingContinuationLines(Vec<u8>),
}

fn cond_output(regex_set : &RegexSet, bytes : &[u8]) -> io::Result<bool> {
    if regex_set.is_match(bytes) {
        io::stdout().write_all(bytes)?;
        Ok (true)
    } else {
        Ok (false)
    }
}

fn search_file(p : &Path, cont_re: &Regex, regex_set : &RegexSet) -> io::Result<bool> {
    use State::*;
    let mut found_match = false;
    let f = File::open(p)?;
    let mut f = io::BufReader::new(f);
    let mut state = ReadingLine;
    loop {
        let mut line = vec![];
        let len = f.read_until(b'\n', &mut line)?;
        match state {
            ReadingLine => {
                if len == 0 {
                    return Ok (found_match)
                }
                state = ReadingContinuationLines(line);
                continue
            }
            ReadingContinuationLines(mut acc) => {
                if len == 0 {
                    found_match |= cond_output(regex_set, &acc)?;
                    return Ok (found_match)
                }
                state = if cont_re.is_match(&line) {
                    acc.write_all(&line)?;
                    ReadingContinuationLines(acc)
                } else {
                    found_match |= cond_output(regex_set, &acc)?;
                    ReadingContinuationLines(line)
                }
            }
        }
    }
}

fn search_files(filepaths : &[&Path], cont_re: &Regex, regex_set : &RegexSet)
                -> io::Result<bool> {
    let mut found_match = false;
    for filepath in filepaths {
        found_match |= search_file(filepath, &cont_re, &regex_set)?;
    }
    Ok (found_match)
}

fn main() {
    let app = App::new("contgrep")
        .version("0.1")
        .arg(Arg::with_name("regex")
             .required(true)
             .short("e")
             .multiple(true)
             .number_of_values(1)
             .value_name("REGEX")
             .help("Match regex"))
        .arg(Arg::with_name("cont_regex")
             .short("c")
             .long("continuation-regex")
             .takes_value(true)
             .multiple(false)
             .value_name("REGEX"))
        .arg(Arg::with_name("files")
             .min_values(1)
             .multiple(true)
             .help("Files to search")
        );
    let matches = app.get_matches();
    let regexes = matches.values_of("regex");
    let filepaths : Vec<_> = matches.values_of("files")
        .unwrap()
        .into_iter()
        .map(|s| {
            Path::new(s)
        })
        .collect();
    let regexes : Vec<&str> = regexes.unwrap().collect();
    let regex_set = RegexSetBuilder::new(&regexes).multi_line(true).build().unwrap();
    let cont_regex = Regex::new(matches.value_of("cont_regex").unwrap_or(r"^\s+")).unwrap();
    match search_files(&filepaths, &cont_regex, &regex_set) {
        Ok (true) => exit(0),
        Ok (false) => exit(1),
        Err (err) => {
            eprintln!("Error: {}", err);
            exit(2)
        }
    }
}
