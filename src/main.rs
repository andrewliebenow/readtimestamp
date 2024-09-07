#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use clap::Parser;
use owo_colors::OwoColorize;
use regex::Regex;
use std::env;
use time::{
    error::ComponentRange, format_description::FormatItem, macros::format_description,
    OffsetDateTime, UtcOffset,
};
use timeago::{Formatter, TimeUnit};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

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
const FORMAT_DESCRIPTION: &[FormatItem<'_>] = format_description!(
    version = 2,
    "[year]-[month]-[day] @ [hour repr:12]:[minute]:[second] [period]"
);
// The largest number that can be parsed by "OffsetDateTime::from_unix_timestamp_nanos" is 253402300799999999999
const MAXIMUM_NUMBER_OF_DIGITS: usize = 21_usize;
const MAXIMUM_NUMBER: i128 = 253_402_300_799_999_999_999_i128;
const MICROSECONDS: &str = "microseconds";
const MILLISECONDS: &str = "milliseconds";
const NANOSECONDS: &str = "nanoseconds";
const SECONDS: &str = "seconds";
const WIDTH: usize = 12_usize;

fn main() -> Result<(), i32> {
    // TODO
    env::set_var("RUST_BACKTRACE", "1");
    // TODO
    env::set_var("RUST_LOG", "debug");

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    let result = start();

    if let Err(er) = result {
        tracing::error!(
            backtrace = %er.backtrace(),
            error = %er,
        );

        return Err(1_i32);
    }

    Ok(())
}

#[expect(clippy::too_many_lines, reason = "Unimportant")]
fn start() -> anyhow::Result<()> {
    const DATA_ARRAY_LEN: usize = 4_usize;

    let readtimestamp_args = ReadtimestampArgs::parse();

    let timestamp = readtimestamp_args.timestamp;

    let (attempting_to_parse_string, attempting_to_parse_string_plain_length) =
        get_attempting_to_parse_string(&timestamp);

    println!(
        "{attempting_to_parse_string}\n{}",
        "-".repeat(attempting_to_parse_string_plain_length)
    );

    let mut has_printed_note = false;

    // Fast path
    let mut timestamp_is_numeric = true;
    let mut number_of_digits = 0_usize;

    for ch in timestamp.chars() {
        if ch.is_ascii_digit() {
            number_of_digits += 1_usize;
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

        // TODO
        // Return code
        return Ok(());
    }

    let str_to_parse = if timestamp_is_numeric {
        &timestamp
    } else {
        eprintln!(
            "{}",
            format!("NOTE: {ARGUMENT_NAME} contains non-digit characters, attempting to find something that looks like a timestamp").yellow()
        );

        has_printed_note = true;

        let regex = Regex::new("[0-9]+")?;

        let mut longest_valid_match = None;
        let mut longest_valid_match_length = 0_usize;
        let mut valid_match_count = 0_u32;

        for ma in regex.find_iter(&timestamp) {
            let ma_len = ma.len();

            if ma_len <= MAXIMUM_NUMBER_OF_DIGITS {
                valid_match_count += 1_u32;

                if ma_len > longest_valid_match_length {
                    longest_valid_match_length = ma_len;
                    longest_valid_match = Some(ma);
                }
            }
        }

        if let Some(ma) = longest_valid_match {
            if valid_match_count > 1_u32 {
                eprintln!(
                    "{}",
                    format!("NOTE: {valid_match_count} possible timestamps were found in {ARGUMENT_NAME}. Parsing the longest one that is not too long to be parsed (if two possible timestamps of the same length were found, the first one will be parsed).").yellow()
                );

                has_printed_note = true;
            }

            let st = ma.as_str();

            // TODO
            #[expect(clippy::format_in_format_args, reason = "Unimportant")]
            {
                eprintln!(
                    "{}",
                    format!(
                        "{}{}{}",
                        format!("Parsing possible timestamp found in {ARGUMENT_NAME}: \"").yellow(),
                        st.bold().yellow(),
                        '"'.yellow()
                    )
                );
            }

            st
        } else {
            eprintln!(
                "{}",
                format!("ERROR: {ARGUMENT_NAME} does not contain any possible timestamps (groups of numbers of the appropriate length)").red());

            // TODO
            // Return code
            return Ok(());
        }
    };

    let str_to_parse_i_six_four = str_to_parse.parse::<i128>();

    match str_to_parse_i_six_four {
        Ok(io) => {
            if io > MAXIMUM_NUMBER {
                eprintln!(
                    "{}",
                    format!("ERROR: Timestamp candidate {io} is too large (greater than {MAXIMUM_NUMBER})").red());

                // TODO
                // Return code
                return Ok(());
            }

            let nanos = Some(OffsetDateTime::from_unix_timestamp_nanos(io));

            let result = i64::try_from(io);

            let (micros_option, millis_option, seconds_option) = if let Ok(is) = result {
                let micros_for_nanos = io * 1_000_i128;

                let millis_for_nanos = micros_for_nanos * 1_000_i128;

                let micros = OffsetDateTime::from_unix_timestamp_nanos(micros_for_nanos);
                let millis = OffsetDateTime::from_unix_timestamp_nanos(millis_for_nanos);
                let seconds = OffsetDateTime::from_unix_timestamp(is);

                (Some(micros), Some(millis), Some(seconds))
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

            // TODO
            #[cfg(debug_assertions)]
            {
                check_width()?;
            }

            let microseconds_str = pad_to_left(WIDTH, MICROSECONDS);
            let milliseconds_str = pad_to_left(WIDTH, MILLISECONDS);
            let nanoseconds_str = pad_to_left(WIDTH, NANOSECONDS);
            let seconds_str = pad_to_left(WIDTH, SECONDS);

            let microseconds_data =
                get_data(&formatter, now_utc, offset, micros_option, microseconds_str)?;
            let milliseconds_data =
                get_data(&formatter, now_utc, offset, millis_option, milliseconds_str)?;
            let nanoseconds_data = get_data(&formatter, now_utc, offset, nanos, nanoseconds_str)?;
            let seconds_data = get_data(&formatter, now_utc, offset, seconds_option, seconds_str)?;

            let data_array: [Data; DATA_ARRAY_LEN] = [
                seconds_data,
                milliseconds_data,
                microseconds_data,
                nanoseconds_data,
            ];

            let mut has_none = Vec::<Data>::with_capacity(DATA_ARRAY_LEN);
            let mut has_some = Vec::<DataWithDelta>::with_capacity(DATA_ARRAY_LEN);

            for da in data_array {
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
                // TODO
                println!();
            }

            let has_none_is_not_empty = !has_none.is_empty();
            let has_some_is_not_empty = !has_some.is_empty();

            if has_some_is_not_empty {
                has_some.sort_by(|da, dat| da.delta.abs().cmp(&dat.delta.abs()));

                for (us, da) in has_some.into_iter().enumerate() {
                    let description = &da.description;
                    let unit = &da.unit;

                    // TODO
                    let is_best_candidate_unit = us == 0_usize;

                    let unit_description = format!("({unit}) {description}");

                    println!(
                        "{}{}{}",
                        if us == 1_usize { "\n" } else { "" },
                        if is_best_candidate_unit {
                            format!("{}\n", "Best candidate unit:".bold().green())
                        } else {
                            String::new()
                        },
                        if is_best_candidate_unit {
                            unit_description.bold().to_string()
                        } else {
                            unit_description
                        }
                    );
                }
            }

            if has_none_is_not_empty {
                if has_some_is_not_empty {
                    // TODO
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

    Ok(())
}

fn get_attempting_to_parse_string(timestamp: &str) -> (String, usize) {
    const PREFIX: &str = "Attempting to parse \"";
    const SUFFIX: &str = "\"";

    // TODO
    // Unchecked arithmetic
    const PREFIX_LEN_PLUS_SUFFIX_LEN: usize = PREFIX.len() + SUFFIX.len();

    let attempting_to_parse_string = format!("{PREFIX}{}{SUFFIX}", timestamp.bold());

    (
        attempting_to_parse_string,
        PREFIX_LEN_PLUS_SUFFIX_LEN + timestamp.len(),
    )
}

fn get_data(
    formatter: &Formatter,
    now_utc: OffsetDateTime,
    offset: Option<UtcOffset>,
    other: Option<Result<OffsetDateTime, ComponentRange>>,
    unit: String,
) -> anyhow::Result<Data> {
    let data = if let Some(re) = other {
        match re {
            Ok(of) => {
                let duration = of - now_utc;

                let date_formatted = of.format(FORMAT_DESCRIPTION)?;

                let local_string = if let Some(ut) = offset {
                    let local = of.to_offset(ut);

                    let local_formatted = local.format(FORMAT_DESCRIPTION)?;

                    format!(" local: {}", local_formatted.purple())
                } else {
                    String::new()
                };

                let duration_unsigned_abs = duration.unsigned_abs();

                let duration_is_positive = duration.is_positive();

                let relative = format!(
                    "{}{}{}",
                    if duration_is_positive { "in " } else { "" },
                    formatter.convert(duration_unsigned_abs),
                    if duration_is_positive { "" } else { " ago" }
                );

                let description = format!(
                    "UTC: {}{local_string} ({})",
                    date_formatted.blue(),
                    relative.cyan(),
                );

                let delta = Some(duration);

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
    };

    Ok(data)
}

fn pad_to_left(width: usize, input: &str) -> String {
    format!("{}{input}", " ".repeat(width - input.len()))
}

#[allow(dead_code, reason = "Unimportant")]
fn check_width() -> anyhow::Result<()> {
    use anyhow::Context;

    const LEN_ARRAY: [usize; 4_usize] = [
        MICROSECONDS.len(),
        MILLISECONDS.len(),
        NANOSECONDS.len(),
        SECONDS.len(),
    ];

    let correct_width = LEN_ARRAY.into_iter().max().context("TODO")?;

    anyhow::ensure!(WIDTH == correct_width);

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_check_width() -> anyhow::Result<()> {
        crate::check_width()
    }
}
