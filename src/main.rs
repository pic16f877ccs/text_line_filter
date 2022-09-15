#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
use clap::{arg, command, value_parser, AppSettings, ArgAction, ArgGroup, ArgMatches};
use colored::{Color, Colorize};
use itertools::Itertools;
use rand::prelude::*;
use std::io::{self, prelude::*};

enum FilterFlags {
    InvertFlag(bool),
    ParseNum(bool),
    ParseFloat(bool),
    ParseAscii(bool),
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

    fn split_start_end(
        &self,
        filter_fns: &FilterFlags,
        delim: &str,
        pat: &Vec<&String>,
    ) -> Vec<&str> {
        use FilterFlags::*;
        self.select_text
            .split(delim)
            .filter(|elem| {
                !pat.iter().any(|c| c.contains(elem))
                    && match filter_fns {
                        ParseNum(flag) => elem.parse::<isize>().is_ok() == *flag,
                        ParseFloat(flag) => elem.parse::<i32>().is_ok() == *flag,
                        ParseAscii(flag) => elem.is_ascii() == *flag,
                        InvertFlag(flag) => *flag,
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

trait IdxFromPos {
    fn start_idx(&self, pos: usize) -> Result<usize, String>;
    fn end_idx(&self, pos: usize) -> Result<usize, String>;
}

impl IdxFromPos for String {
    fn start_idx(&self, char_pos: usize) -> Result<usize, String> {
        if char_pos == 0 || self.chars().count() < char_pos {
            return Err("Not position".to_string());
        }
        let mut index = 0;
        let mut prev_index = 0;
        Ok(self
            .chars()
            .map(|chr| {
                index += prev_index;
                prev_index = chr.len_utf8();
                index
            })
            .nth(char_pos - 1)
            .unwrap())
    }

    fn end_idx(&self, char_pos: usize) -> Result<usize, String> {
        if char_pos == 0 || self.chars().count() < char_pos {
            return Err("Not position".to_string());
        }
        let mut index = 0;
        Ok(self
            .chars()
            .map(|chr| {
                index += chr.len_utf8();
                index
            })
            .nth(char_pos - 1)
            .unwrap())
    }
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

fn set_color(color: &str) -> Color {
    match color {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        &_ => todo!(),
    }
}

fn main() -> io::Result<()> {
    let mut delim = "";
    let mut start_end_pat = ("", "");
    let mut pos: (usize, usize) = (0, 0);
    let mut filter_flags: FilterFlags = FilterFlags::InvertFlag(true);
    let mut select_len: usize = 0;
    let mut rng = rand::thread_rng();
    let app_cmmd = app_commands();

    let mut color_value: &str = app_cmmd
        .get_one::<String>("color")
        .expect("Defaul 'value none'");
    let mut stdin_line = String::new();
    io::stdout().flush()?;

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
        let flag: bool = *app_cmmd
            .get_one("number")
            .expect("Internal error, --number <BOOL> 'value none'");
        filter_flags = FilterFlags::ParseNum(flag);
    }

    if app_cmmd.is_present("float") {
        let flag: bool = *app_cmmd
            .get_one("float")
            .expect("Internal error, --float <BOOL> 'value none'");
        filter_flags = FilterFlags::ParseFloat(flag);
    }

    if app_cmmd.is_present("ascii") {
        let flag: bool = *app_cmmd
            .get_one("ascii")
            .expect("Internal error, --ascii <BOOL> 'value none'");
        filter_flags = FilterFlags::ParseAscii(flag);
    }

    if app_cmmd.is_present("invert") {
        let flag: bool = *app_cmmd
            .get_one("invert")
            .expect("Internal error, --invert <BOOL> 'value none'");
        filter_flags = FilterFlags::InvertFlag(flag);
    }

    let exclude_vals = if app_cmmd.contains_id("exclude") {
        app_cmmd
            .get_many::<String>("exclude")
            .expect("Error argumet exclude")
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    if app_cmmd.is_present("delimiter") {
        delim = *app_cmmd
            .get_one("delimiter")
            .expect("Internal error, --delimiter <STRING> 'value none'");
    }
    if app_cmmd.is_present("color") {
        color_value = app_cmmd
            .get_one::<String>("color")
            .expect("Internal error, --color <STRING> 'value none'");
    }

    for line in io::stdin().lines() {
        let mut vec_range: Vec<String> = Vec::new();
        let mut vec: Vec<&str> = Vec::new();
        stdin_line = line?;

        if !(stdin_line.is_empty()) {
            if app_cmmd.contains_id("selection") {
                if app_cmmd.is_present("selected") {
                    start_end_pat = app_cmmd
                        .get_many("selected")
                        .expect("Internal error, --selected <STRING> 'value none'")
                        .copied()
                        .collect_tuple()
                        .expect("Internal error, --selected <STRING> 'tuple create'");
                    start_end_len = stdin_line.find_start_end(start_end_pat);
                } else if app_cmmd.is_present("position") {
                    let start_stop: (usize, usize) = app_cmmd
                        .get_many("position")
                        .expect("Internal error, --position <VALUE> 'value none'")
                        .copied()
                        .collect_tuple()
                        .expect("Internal error, --position <VALUE> 'tuple create'");
                    match stdin_line.start_idx(start_stop.0) {
                        Ok(value) => {
                            start_end_len.start = value;
                        }
                        Err(_) => {
                            start_end_len.start = stdin_line.len();
                        }
                    }

                    match stdin_line.end_idx(start_stop.1) {
                        Ok(value) => {
                            start_end_len.end = value;
                        }
                        Err(_) => {
                            start_end_len.end = stdin_line.len();
                        }
                    }
                }
                start_end_len.len = stdin_line.len();
                selected_text =
                    SelectedText::new(&stdin_line, start_end_len.start, start_end_len.end);
                vec = selected_text.split_start_end(&filter_flags, delim, &exclude_vals);
            }
        }

        if app_cmmd.is_present("range") {
            vec_range = range_arg(&app_cmmd);
            vec = vec_range.iter().map(|elem| &**elem).collect();
        }

        if app_cmmd.is_present("shuffle") {
            vec.shuffle(&mut rng);
        }

        if app_cmmd.is_present("separator") {
            delim = *app_cmmd
                .get_one("separator")
                .expect("Internal error, --separator <STRING> 'value none'");
        }

        if app_cmmd.is_present("hide") {
            start_end_len.start = 0;
            start_end_len.len = start_end_len.end;
        }

        println!(
            "{}{}{}",
            &stdin_line[0..start_end_len.start],
            &vec.join(&delim).color(set_color(color_value)),
            &stdin_line[start_end_len.end..start_end_len.len]
        );
    }

    Ok(())
}

fn app_commands() -> ArgMatches {
    command!()
        .global_setting(AppSettings::DeriveDisplayOrder)
        .about("    vim external filter command")
        .author("    by PIC16F877ccs")
        .args_override_self(true)
        .arg(
            arg!(-n  --number <BOOL>         "Number char or not number filter")
                .number_of_values(1)
                .value_parser(value_parser!(bool))
                .default_missing_value("true")
                .required(false),
        )
        .arg(
            arg!(-f  --float <BOOL>        "Float char or not float filter")
                .number_of_values(1)
                .value_parser(value_parser!(bool))
                .default_missing_value("true")
                .required(false),
        )
        .arg(
            arg!(-A  --ascii <BOOL>        "Ascii char or no ascii, filter")
                .number_of_values(1)
                .value_parser(value_parser!(bool))
                .default_missing_value("true")
                .required(false),
        )
        .arg(
            arg!(-i  --invert <BOOL>        "Invert selection filter")
                .number_of_values(1)
                .value_parser(value_parser!(bool))
                .default_missing_value("true")
                .required(false),
        )
        .arg(
            arg!(-e  --exclude <STRING>        "Exclude chars filter")
                .action(ArgAction::Append)
                .takes_value(true)
                .required(false),
        )
        .arg(
            arg!(-d  --delimiter <STRING>        "Text delimiter")
                .number_of_values(1)
                .required(false),
        )
        .arg(
            arg!(-D  --separator <STRING>        "Output text delimiter")
                .number_of_values(1)
                .required(false),
        )
        .arg(
            arg!(-C  --color <STRING>        "Highlight color")
                .number_of_values(1)
                .value_parser([
                    "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
                ])
                .default_value("black")
                .required(false),
        )
        .arg(
            arg!(-s  --selected <STRING>       "Selected text start end")
                .number_of_values(2)
                .required(false),
        )
        .arg(
            arg!(-p  --position <NUMBER>      "Select start and end characters")
                .value_parser(value_parser!(usize))
                .number_of_values(2)
                .use_value_delimiter(true)
                .require_value_delimiter(true)
                .multiple_values(true)
                .required(false),
        )
        .arg(arg!(-S  --shuffle       "Selected text or range shuffle").required(false))
        .arg(arg!(-c  --clear       "Line text clear").required(false))
        .arg(arg!(-h  --hide        "Hide unselected text").required(false))
        .arg(
            arg!(-r  --range <NUMBER>      "Insert a range in the selected text")
                .value_parser(value_parser!(isize))
                .number_of_values(2)
                .use_value_delimiter(true)
                .require_value_delimiter(true)
                .multiple_values(true)
                .requires("selection")
                .required(false),
        )
        .group(ArgGroup::new("filter_type").args(&["invert", "number", "float", "ascii"]))
        .group(ArgGroup::new("selection").args(&["position", "selected"]))
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

