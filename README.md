# `readtimestamp`

I frequently use Unix timestamps as, or in, file and directory names when I need a quick ~guaranteed unique name (usually I use `date +%s` for this). This works well, but to later check when the directory or file was created, I have to copy the Unix timestamp and use a tool or website to convert it to something more human readable. `readtimestamp` is a simple tool that takes one argument and prints in a human readable format the first substring in that argument that looks like a Unix timestamp. Since some programs generate millisecond or nanosecond timestamps (e.g. Java's `System.currentTimeMillis()`), `readtimestamp` interprets the timestamp substring in multiple units, printing the most probable candidate first. See below for an example.

## Installation

```Shell
# TODO Publish to crates.io
cargo install --git https://github.com/andrewliebenow/readtimestamp
```

## Usage

```Shell
❯ readtimestamp --help
Pretty print a Unix timestamp (seconds or milliseconds)

Usage: readtimestamp <TIMESTAMP>

Arguments:
  <TIMESTAMP>  The Unix timestamp to parse (can be in seconds, milliseconds, or nanoseconds)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```Shell
❯ readtimestamp ./my-saved-file-1704772140
Attempting to parse "./my-saved-file-1704772140"
------------------------------------------------
NOTE: <TIMESTAMP> contains non-digit characters, attempting to find something that looks like a timestamp
Parsing possible timestamp found in <TIMESTAMP>: "1704772140"

Best candidate unit:
(     seconds) UTC: 2024-01-09 @ 03:49:00 AM local: 2024-01-08 @ 10:49:00 PM (7 months 4 weeks 1 day 11 hours 25 minutes ago)

(milliseconds) UTC: 1970-01-20 @ 05:32:52 PM local: 1970-01-20 @ 12:32:52 PM (54 years 8 months 11 hours 9 minutes 14 seconds ago)
(microseconds) UTC: 1970-01-01 @ 12:28:24 AM local: 1969-12-31 @ 07:28:24 PM (54 years 8 months 2 weeks 6 days 4 hours ago)
( nanoseconds) UTC: 1970-01-01 @ 12:00:01 AM local: 1969-12-31 @ 07:00:01 PM (54 years 8 months 2 weeks 6 days 4 hours ago)
```

## License

MIT License, see <a href="LICENSE">LICENSE</a> file
