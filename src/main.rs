extern crate clap;
extern crate regex;
extern crate subparse;

use clap::{Command, Arg};
use regex::Regex;

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use subparse::timetypes::{TimeDelta, TimePoint, TimeSpan};
use subparse::{get_subtitle_format, parse_str};
use subparse::{SrtFile, SubtitleEntry, SubtitleFileInterface, SubtitleFormat};

/// This function reads the content of a file to a `String`.
fn read_file(path: &Path) -> String {
    use std::io::Read;
    let mut file = std::fs::File::open(path).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    s
}

fn parse_timestamp(input: &str) -> TimeDelta {
    let re = Regex::new(r"([+-]?)(\d{2}):(\d{2}):(\d{2}),(\d{3})").unwrap();
    if !re.is_match(input) {
        panic!("Could not parse timestamp");
    }

    let cap = re.captures(input).unwrap();

    let sign = if String::from(&cap[1]) == "-" { -1 } else { 1 };
    let h = 60 * 60 * 1000 * String::from(&cap[2]).parse::<i64>().unwrap();
    let m = 60 * 1000 * String::from(&cap[3]).parse::<i64>().unwrap();
    let s = 1000 * String::from(&cap[4]).parse::<i64>().unwrap();
    let ms = String::from(&cap[5]).parse::<i64>().unwrap();

    let ms_total = h + m + s + ms;

    return TimeDelta::from_msecs(sign * ms_total);
}

fn shift_entries(
    entries: Vec<SubtitleEntry>,
    delta: TimeDelta,
    end: TimePoint,
) -> Vec<SubtitleEntry> {
    let mut entries_new = Vec::new();

    for subtitle_entry in entries {
        let timespan = subtitle_entry.timespan + delta;

        // Skip negative times and after end entries
        if timespan.start.is_negative() || subtitle_entry.timespan.end >= end {
            continue;
        }

        entries_new.push(SubtitleEntry {
            timespan: timespan,
            line: subtitle_entry.line,
        });
    }

    return entries_new;
}

fn write_srt_file(output_path: &str, entries: Vec<SubtitleEntry>) {
    let path = Path::new(output_path);

    let mut file = match File::create(&path) {
        Err(why) => panic!("Couldn't create file {}: {}", output_path, why.to_string()),
        Ok(file) => file,
    };

    let mut parts: Vec<(TimeSpan, String)> = Vec::new();

    for entry in entries {
        parts.push((entry.timespan, entry.line.unwrap_or(String::new())));
    }

    let srt_contents = String::from_utf8(
        SrtFile::create(parts)
            .expect("Could not create srt file")
            .to_data()
            .unwrap(),
    )
    .unwrap();

    match file.write_all(srt_contents.as_bytes()) {
        Err(why) => panic!("Couldn't write to {}: {}", output_path, why.to_string()),
        Ok(_) => println!("Successfully wrote to {}", output_path),
    }
}

fn main() {
    let matches = Command::new("srtshift")
        .version("0.1.1")
        .author("Jonas Karlsson <jonaskarlsson@fripost.org>")
        .about("Shift and trim SubRip files")
        .allow_hyphen_values(true)
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .takes_value(true)
                .required(true)
                .help("Input file"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .takes_value(true)
                .required(true)
                .help("Output file"),
        )
        .arg(
            Arg::new("shift")
                .short('s')
                .long("shift")
                .takes_value(true)
                .required(true)
                .help("Shift timestamps [+/-]hh:mm:ss,xxx"),
        )
        .arg(
            Arg::new("cutafter")
                .short('a')
                .long("cutafter")
                .takes_value(true)
                .help("Cut timestamps after [+/-]hh:mm:ss,xxx"),
        )
        .get_matches();

    let shift = parse_timestamp(matches.value_of("shift").unwrap());
    let cutafter = parse_timestamp(matches.value_of("cutafter").unwrap_or("99:99:99,999"));

    let path = PathBuf::from(matches.value_of("input").unwrap());
    let output_path = matches.value_of("output").unwrap();
    let file_content: String = read_file(&path);

    let format =
        get_subtitle_format(path.extension(), file_content.as_bytes()).expect("unknown format");

    if format != SubtitleFormat::SubRip {
        panic!("Only srt files supported.");
    }

    let subtitle_file = parse_str(format, &file_content, 25.0).expect("parser error");
    let subtitle_entries: Vec<SubtitleEntry> = subtitle_file
        .get_subtitle_entries()
        .expect("unexpected error");

    let end = TimePoint::from_msecs(cutafter.msecs());
    let entries_new = shift_entries(subtitle_entries, shift, end);

    write_srt_file(output_path, entries_new);
}
