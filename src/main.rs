#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
use clap::{arg, command, value_parser, AppSettings, ArgAction, ArgGroup, ArgMatches};
use itertools::Itertools;
use rand::prelude::*;
use std::io::{self, prelude::*};

enum FilterFns {
    TrueFn,
    ParseNumFn,
    ParseFloatFn,
    NotParseNumFn,
}

#[derive(Debug)]
struct SelectedText<'a> {
    start: usize,
    end: usize,
    select_text: &'a str,
}

impl<'a> SelectedText<'a> {
    fn new(string: &'a String, start: usize, end: usize) -> Self {
        SelectedText {
            start,
            end,
            select_text: &string[start..end],
        }
    }

    fn split_start_end(&self, filter_fns: FilterFns, delim: &str, pat: Vec<&String>) -> Vec<&str> {
        use FilterFns::*;
        self.select_text
            .split(delim)
            .filter(|elem| {
                !pat.iter().any(|c| c.contains(elem))
                    && match filter_fns {
                        ParseNumFn => elem.parse::<isize>().is_ok(),
                        ParseFloatFn => elem.parse::<i32>().is_ok(),
                        NotParseNumFn => !elem.parse::<isize>().is_ok(),
                        _ => true,
                    }
            })
            .collect()
    }
}

struct StartEndLen {
    start: usize,
    end: usize,
    len: usize,
}

trait StartEnd {
    fn find_start_end(&self, pat_start_end: (&str, &str)) -> StartEndLen;
}

impl StartEnd for String {
    // find start and end for a range in a string
    fn find_start_end(&self, pat_start_end: (&str, &str)) -> StartEndLen {
        if let Some(start) = self.find(pat_start_end.0) {
            let end = if let Some(end) = self.find(pat_start_end.1) {
                end
            } else {
                return StartEndLen {
                    start: 0,
                    end: 0,
                    len: 0,
                };
            };
            if start > end {
                return StartEndLen {
                    start: 0,
                    end: 0,
                    len: 0,
                };
            }
            StartEndLen {
                start,
                end: end + pat_start_end.1.chars().count(),
                len: self.len(),
            }
        } else {
            return StartEndLen {
                start: 0,
                end: 0,
                len: 0,
            };
        }
    }
}

fn main() -> io::Result<()> {
    let mut delim = "";
    let mut start_end_pat = ("", "");
    let mut pos: (usize, usize) = (0, 1);
    let mut filter_fns: FilterFns = FilterFns::TrueFn;
    let mut rng = rand::thread_rng();
    let mut vec: Vec<&str> = Vec::new();
    let app_cmmd = app_commands();
    let mut vec_range: Vec<String> = Vec::new();

    let mut stdin_line = String::new();
    io::stdout().flush()?;
    io::stdin().read_line(&mut stdin_line)?;

    let mut start_end_len = StartEndLen {
        start: 0,
        end: 0,
        len: 0,
    };

    let mut selected_text = SelectedText {
        start: 0,
        end: 0,
        select_text: "",
    };

    if app_cmmd.is_present("clear") {
        stdin_line.clear();
    }

    if app_cmmd.is_present("number") {
        filter_fns = FilterFns::ParseNumFn;
    }

    if app_cmmd.is_present("float") {
        filter_fns = FilterFns::ParseFloatFn;
    }

    if app_cmmd.is_present("alpha") {
        filter_fns = FilterFns::NotParseNumFn;
    }

    let exclude_vals = if app_cmmd.contains_id("exclude") {
        app_cmmd
            .get_many::<String>("exclude")
            .expect("Error argumet exclude")
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    if !(stdin_line.is_empty()) {
        if app_cmmd.is_present("selected") {
            start_end_pat = app_cmmd
                .values_of("selected")
                .expect("Internal error, --selected <STRING> 'value none'")
                .collect_tuple()
                .expect("Internal error, --selected <STRING> 'tuple create'");
            start_end_len = stdin_line.find_start_end(start_end_pat);
            start_end_len.len = stdin_line.len();
            selected_text = SelectedText::new(&stdin_line, start_end_len.start, start_end_len.end);
            vec = selected_text.split_start_end(filter_fns, delim, exclude_vals);
        }
    }

    if app_cmmd.is_present("delimiter") {
        delim = app_cmmd
            .value_of("delimiter")
            .expect("Internal error, --delimiter <STRING> 'value none'");
    }

    if app_cmmd.is_present("range") {
        vec_range = range_arg(&app_cmmd);
        vec = vec_range.iter().map(|elem| &**elem).collect();
    }

    if app_cmmd.is_present("shuffle") {
        vec.shuffle(&mut rng);
    }

    if app_cmmd.is_present("separator") {
        delim = app_cmmd
            .value_of("separator")
            .expect("Internal error, --separator <STRING> 'value none'");
    }

    print!(
        "{}{}{}",
        &stdin_line[0..start_end_len.start],
        &vec.join(&delim),
        &stdin_line[start_end_len.end..start_end_len.len]
    );

    Ok(())
}

fn app_commands() -> ArgMatches {
    command!()
        .global_setting(AppSettings::DeriveDisplayOrder)
        .about("    vim external filter command")
        .author("    by PIC16F877ccs")
        .args_override_self(true)
        .arg(arg!(-n  --number        "Number char filter").required(false))
        .arg(arg!(-f  --float        "Float char filter").required(false))
        .arg(arg!(-a  --alpha        "Not number char filter").required(false))
        .arg(
            arg!(-d  --delimiter <STRING>        "Text delimiter")
                .number_of_values(1)
                .required(false),
        )
        .arg(
            arg!(-e  --exclude <STRING>        "Exclude chars filter")
                .action(ArgAction::Append)
                .short('e')
                .takes_value(true)
                .required(false),
        )
        .arg(
            arg!(-D  --separator <STRING>        "Output text delimiter")
                .number_of_values(1)
                .required(false),
        )
        .arg(
            arg!(-s  --selected <STRING>       "selected text start end")
                .number_of_values(2)
                .required(false),
        )
        .arg(arg!(-S  --shuffle       "Highlight text or range shuffle").required(false))
        .arg(arg!(-c  --clear       "Line text clear").required(false))
        .arg(
            arg!(-r  --range <NUMBER>      "Insert a range in the selected text")
                .value_parser(value_parser!(isize))
                .number_of_values(2)
                .use_value_delimiter(true)
                .require_value_delimiter(true)
                .multiple_values(true)
                .required(false),
        )
        .group(ArgGroup::new("filter_type").args(&["number", "float", "alpha"]))
        .get_matches()
}

fn range_arg(app_cmmd: &ArgMatches) -> Vec<String> {
    let mut start_stop = (0_isize, 0_isize);
    start_stop = app_cmmd
        .get_many("range")
        .expect("Internal error, --range <VALUE> 'value none'")
        .copied()
        .collect_tuple()
        .expect("Internal error, --range <VALUE> 'tuple create'");
    (start_stop.0..start_stop.1)
        .map(|var| var.to_string())
        .collect::<Vec<_>>()
}
