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

struct FormatOptions {
    file_prefix : bool,
    line_numbers : bool,
}

struct FormatCtx<'a, 'b> {
    opts : &'a FormatOptions,
    path : &'b Path,
    line_nr : usize,
}

fn format_append_line(acc : &mut io::Write, fctx : &FormatCtx, bytes : &[u8]) -> io::Result<()> {
    if fctx.opts.file_prefix {
        write!(acc, "{}:", fctx.path.display())?;
    }
    if fctx.opts.line_numbers {
        write!(acc, "{}:", fctx.line_nr)?;
    }
    acc.write_all(bytes)
}

fn format_create_line(fctx : &FormatCtx, bytes : &[u8]) -> io::Result<Vec<u8>> {
    let mut buf = vec![];
    format_append_line(&mut buf, fctx, bytes)?;
    Ok (buf)
}

fn search_file(p : &Path, cont_re: &Regex, regex_set : &RegexSet,
               fmtopts : &FormatOptions) -> io::Result<bool> {
    use State::*;
    let mut fctx = FormatCtx {
        opts : fmtopts,
        path : p,
        line_nr : 0,
    };
    let mut found_match = false;
    let f = File::open(p)?;
    let mut f = io::BufReader::new(f);
    let mut state = ReadingLine;
    loop {
        let mut line = vec![];
        fctx.line_nr += 1;
        let len = f.read_until(b'\n', &mut line)?;
        match state {
            ReadingLine => {
                if len == 0 {
                    return Ok (found_match)
                }
                let line = format_create_line(&fctx, &line)?;
                state = ReadingContinuationLines(line);
                continue
            }
            ReadingContinuationLines(mut acc) => {
                if len == 0 {
                    found_match |= cond_output(regex_set, &acc)?;
                    return Ok (found_match)
                }
                state = if cont_re.is_match(&line) {
                    format_append_line(&mut acc, &fctx, &line)?;
                    ReadingContinuationLines(acc)
                } else {
                    found_match |= cond_output(regex_set, &acc)?;
                    let line = format_create_line(&fctx, &line)?;
                    ReadingContinuationLines(line)
                }
            }
        }
    }
}

fn search_files(filepaths : &[&Path], cont_re: &Regex, regex_set : &RegexSet,
                fmtopts : FormatOptions) -> io::Result<bool> {
    let mut found_match = false;
    for filepath in filepaths {
        found_match |= search_file(filepath, &cont_re, &regex_set, &fmtopts)?;
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
        .arg(Arg::with_name("with_filename")
             .short("H")
             .required(false)
             .takes_value(false)
             .help("Prepend filename"))
        .arg(Arg::with_name("with_line_number")
             .short("n")
             .required(false)
             .takes_value(false)
             .help("Prepend line number"))
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
    let fmtopts = FormatOptions {
        file_prefix : matches.occurrences_of("with_filename") > 0 ||
            filepaths.len() > 1,
        line_numbers : matches.occurrences_of("with_line_number") > 0,
    };
    match search_files(&filepaths, &cont_regex, &regex_set, fmtopts) {
        Ok (true) => exit(0),
        Ok (false) => exit(1),
        Err (err) => {
            eprintln!("Error: {}", err);
            exit(2)
        }
    }
}
