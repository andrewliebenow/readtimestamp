#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use clap::Parser;

use owo_colors::OwoColorize;
use regex::Regex;
use time::{error::ComponentRange, macros::format_description, OffsetDateTime, UtcOffset};
use timeago::{Formatter, TimeUnit};

/// Pretty print a Unix timestamp (seconds or milliseconds)
#[derive(Parser)]
#[command(author, version, about)]
struct ReadtimestampArgs {
    /// The Unix timestamp to parse (can be in seconds, milliseconds, or nanoseconds)
    timestamp: String,
}

struct Data {
    delta: Option<time::Duration>,
    description: String,
    unit: String,
}

struct DataWithDelta {
    delta: time::Duration,
    description: String,
    unit: String,
}

const ARGUMENT_NAME: &str = "<TIMESTAMP>";
const FORMAT_DESCRIPTION: &[time::format_description::FormatItem<'_>] = format_description!(
    version = 2,
    "[year]-[month]-[day] @ [hour repr:12]:[minute]:[second] [period]"
);
const MAXIMUM_NUMBER_OF_DIGITS: usize = 21;
const MICROSECONDS: &str = "microseconds";
const MILLISECONDS: &str = "milliseconds";
const NANOSECONDS: &str = "nanoseconds";
const SECONDS: &str = "seconds";

fn pad_to_left(width: usize, input: &str) -> String {
    format!("{}{input}", " ".repeat(width - input.len()))
}

#[allow(clippy::too_many_lines)]
fn main() {
    // Use "build.rs"?
    let pad_to_length = [SECONDS, MILLISECONDS, MICROSECONDS, NANOSECONDS]
        .iter()
        .map(|st| st.len())
        .max()
        .unwrap();

    let microseconds_str = pad_to_left(pad_to_length, MICROSECONDS);
    let milliseconds_str = pad_to_left(pad_to_length, MILLISECONDS);
    let nanoseconds_str = pad_to_left(pad_to_length, NANOSECONDS);
    let seconds_str = pad_to_left(pad_to_length, SECONDS);

    let readtimestamp_args = ReadtimestampArgs::parse();

    let timestamp = readtimestamp_args.timestamp;

    let attempting_to_parse_string = format!("Attempting to parse \"{}\"", timestamp.bold());

    println!(
        "{attempting_to_parse_string}\n{}",
        "-".repeat(attempting_to_parse_string.len())
    );

    let mut has_printed_note = false;

    // Fast path
    let mut timestamp_is_numeric = true;
    let mut number_of_digits = 0;

    for ch in timestamp.chars() {
        if ch.is_ascii_digit() {
            number_of_digits += 1;
        } else {
            timestamp_is_numeric = false;

            break;
        }
    }

    if timestamp_is_numeric && number_of_digits > MAXIMUM_NUMBER_OF_DIGITS {
        eprintln!(
            "{}",
            format!(
                "ERROR: {ARGUMENT_NAME} is too long (more than {MAXIMUM_NUMBER_OF_DIGITS} digits)"
            )
            .red()
        );

        return;
    }

    let str_to_parse = if timestamp_is_numeric {
        &timestamp
    } else {
        eprintln!(
            "{}",
            format!("NOTE: {ARGUMENT_NAME} contains non-digit characters, attempting to find something that looks like a timestamp").yellow()
        );

        has_printed_note = true;

        let regex = Regex::new("[0-9]+").unwrap();

        let mut longest_valid_match = None;
        let mut longest_valid_match_length = 0;
        let mut valid_match_count = 0;

        for ma in regex.find_iter(&timestamp) {
            let ma_len = ma.len();

            if ma_len <= MAXIMUM_NUMBER_OF_DIGITS {
                valid_match_count += 1;

                if ma_len > longest_valid_match_length {
                    longest_valid_match_length = ma_len;
                    longest_valid_match = ma.into();
                }
            }
        }

        if let Some(ma) = longest_valid_match {
            if valid_match_count > 1 {
                eprintln!(
                    "{}",
                    format!("NOTE: {valid_match_count} possible timestamps were found in {ARGUMENT_NAME}. Parsing the longest one that is not too long to be parsed (if two possible timestamps of the same length were found, the first one will be parsed).").yellow()
                );

                has_printed_note = true;
            }

            ma.as_str()
        } else {
            eprintln!(
                "{}",
                format!("ERROR: {ARGUMENT_NAME} does not contain any possible timestamps (groups of numbers of the appropriate length)").red());

            return;
        }
    };

    let str_to_parse_i_six_four = str_to_parse.parse::<i128>();

    match str_to_parse_i_six_four {
        Ok(io) => {
            let nanos = OffsetDateTime::from_unix_timestamp_nanos(io).into();

            let result: Result<i64, _> = io.try_into();

            let (micros, millis, seconds) = if let Ok(is) = result {
                let micros_for_nanos = io * 1_000;

                let millis_for_nanos = micros_for_nanos * 1_000;

                let micros = OffsetDateTime::from_unix_timestamp_nanos(micros_for_nanos);
                let millis = OffsetDateTime::from_unix_timestamp_nanos(millis_for_nanos);
                let seconds = OffsetDateTime::from_unix_timestamp(is);

                (micros.into(), millis.into(), seconds.into())
            } else {
                (None, None, None)
            };

            let now_utc = OffsetDateTime::now_utc();

            let result = UtcOffset::current_local_offset();

            let offset = match result {
                Ok(ut) => Some(ut),
                Err(ind) => {
                    eprintln!(
                        "{}",
                        format!("NOTE: Could not determine current time zone offset. Dates will only be displayed in UTC. Error reported: \"{ind}\".").yellow()
                    );

                    has_printed_note = true;

                    None
                }
            };

            let formatter = {
                let mut fo = Formatter::new();

                fo.ago("");
                fo.min_unit(TimeUnit::Milliseconds);
                fo.num_items(5);

                fo
            };

            let microseconds_data =
                get_data(&formatter, &now_utc, offset, micros, microseconds_str);
            let milliseconds_data =
                get_data(&formatter, &now_utc, offset, millis, milliseconds_str);
            let nanoseconds_data = get_data(&formatter, &now_utc, offset, nanos, nanoseconds_str);
            let seconds_data = get_data(&formatter, &now_utc, offset, seconds, seconds_str);

            let mut has_none: Vec<Data> = vec![];
            let mut has_some: Vec<DataWithDelta> = vec![];

            for da in [
                seconds_data,
                milliseconds_data,
                microseconds_data,
                nanoseconds_data,
            ] {
                if let Some(du) = da.delta {
                    has_some.push(DataWithDelta {
                        delta: du,
                        description: da.description,
                        unit: da.unit,
                    });
                } else {
                    has_none.push(da);
                }
            }

            if has_printed_note {
                println!();
            }

            let has_none_is_not_empty: bool = !has_none.is_empty();
            let has_some_is_not_empty: bool = !has_some.is_empty();

            if has_some_is_not_empty {
                has_some.sort_by(|an, ano| an.delta.abs().cmp(&ano.delta.abs()));

                for (us, da) in has_some.iter().enumerate() {
                    let description = &da.description;
                    let unit = &da.unit;

                    let is_best_candidate_unit = us == 0;

                    println!(
                        "{}{}{}",
                        if us == 1 { "\n" } else { "" },
                        if is_best_candidate_unit {
                            format!("{}\n", "Best candidate unit:".bold().green())
                        } else {
                            String::new()
                        },
                        if is_best_candidate_unit {
                            format!("({unit}) {description}").bold().to_string()
                        } else {
                            format!("({unit}) {description}")
                        }
                    );
                }
            }

            if has_none_is_not_empty {
                if has_some_is_not_empty {
                    println!();
                }

                for da in has_none {
                    println!("({}) {}", da.unit, da.description);
                }
            }
        }
        Err(pa) => {
            eprintln!(
                "{}",
                format!("ERROR: Could not parse possible timestamp into a 128 bit signed integer. The number is probably too long. Error reported: \"{pa}\".").red()
            );
        }
    }
}

fn get_data(
    formatter: &Formatter,
    now_utc: &OffsetDateTime,
    offset: Option<UtcOffset>,
    other: Option<Result<OffsetDateTime, ComponentRange>>,
    unit: String,
) -> Data {
    if let Some(re) = other {
        match re {
            Ok(of) => {
                let duration: time::Duration = of - *now_utc;

                let date_formatted = of.format(FORMAT_DESCRIPTION).unwrap();

                let local_string = if let Some(ut) = offset {
                    let local = of.to_offset(ut);

                    let local_formatted = local.format(FORMAT_DESCRIPTION).unwrap();

                    format!(" local: {}", local_formatted.purple())
                } else {
                    String::new()
                };

                let yyy = duration.unsigned_abs();

                let duration_is_positive = duration.is_positive();

                let relative = format!(
                    "{}{}{}",
                    if duration_is_positive { "in " } else { "" },
                    formatter.convert(yyy),
                    if duration_is_positive { "" } else { " ago" }
                );

                let description = format!(
                    "UTC: {}{local_string} ({})",
                    date_formatted.blue(),
                    relative.cyan(),
                );

                let delta = duration.into();

                Data {
                    delta,
                    description,
                    unit,
                }
            }
            Err(co) => Data {
                description: format!("error reported: \"{co}\""),
                delta: None,
                unit,
            },
        }
    } else {
        Data {
            description: format!("error: number was too large to interpret as {unit}"),
            delta: None,
            unit,
        }
    }
}
