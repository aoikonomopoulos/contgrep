extern crate clap;
extern crate regex;

use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

use clap::{App, Arg};
use regex::bytes::{Regex, RegexSet, RegexSetBuilder};

enum State {
    ReadingLine,
    ReadingContinuationLines(Vec<u8>),
}

fn cond_output(regex_set : &RegexSet, bytes : &[u8]) -> io::Result<()> {
    if regex_set.is_match(bytes) {
        io::stdout().write_all(bytes)
    } else {
        Ok (())
    }
}

fn search_file(p : &Path, cont_re: &Regex, regex_set : &RegexSet) -> io::Result<()> {
    use State::*;
    let f = File::open(p)?;
    let mut f = io::BufReader::new(f);
    let mut state = ReadingLine;
    loop {
        let mut line = vec![];
        let len = f.read_until(b'\n', &mut line)?;
        match state {
            ReadingLine => {
                if len == 0 {
                    return Ok (())
                }
                state = ReadingContinuationLines(line);
                continue
            }
            ReadingContinuationLines(mut acc) => {
                if len == 0 {
                    cond_output(regex_set, &acc)?;
                    return Ok (())
                }
                state = if cont_re.is_match(&line) {
                    acc.write_all(&line)?;
                    ReadingContinuationLines(acc)
                } else {
                    cond_output(regex_set, &acc)?;
                    ReadingContinuationLines(line)
                }
            }
        }
    }
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
        .arg(Arg::with_name("file")
             .required(true)
             .index(1)
             .help("File to search")
        );
    let matches = app.get_matches();
    let regexps = matches.values_of("regex");
    let filepath = Path::new(matches.value_of("file").unwrap());

    let regexps : Vec<&str> = regexps.unwrap().collect();
    let regexp_set = RegexSetBuilder::new(&regexps).multi_line(true).build().unwrap();
    search_file(&filepath, &Regex::new(r"^\s+").unwrap(), &regexp_set).unwrap()
}
