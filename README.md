# `readtimestamp`

I frequently use Unix timestamps as, or in, file and directory names when I need a quick ~guaranteed unique name (usually I use `date +%s` for this). This works well, but to later check when the directory or file was created, I have to copy the Unix timestamp and use a tool or website to convert it to something more human readable. `readtimestamp` is a simple tool that takes one argument and prints in a human readable format the first substring in that argument that looks like a Unix timestamp. Since some programs generate millisecond or nanosecond timestamps (e.g. Java's `System.currentTimeMillis()`), `readtimestamp` interprets the timestamp substring in multiple units, printing the most probable candidate first. See below for an example.

## Installation

```
# TODO To be published to crates.io
git clone https://github.com/andrewlabors/readtimestamp && cargo install --path ./readtimestamp
```

## Usage

```shell
❯ readtimestamp ./my-saved-file-1704772140
Attempting to parse "./my-saved-file-1704772140"
--------------------------------------------------------
NOTE: <TIMESTAMP> contains non-digit characters, attempting to find something that looks like a timestamp

Best candidate unit:
(     seconds) UTC: 2024-01-09 @ 03:49:00 AM local: 2024-01-08 @ 09:49:00 PM (1 month 5 days 14 hours 1 minute 47 seconds ago)

(milliseconds) UTC: 1970-01-20 @ 05:32:52 PM local: 1970-01-20 @ 11:32:52 AM (54 years 1 month 6 days 23 hours 45 minutes ago)
(microseconds) UTC: 1970-01-01 @ 12:28:24 AM local: 1969-12-31 @ 06:28:24 PM (54 years 1 month 3 weeks 5 days 16 hours ago)
( nanoseconds) UTC: 1970-01-01 @ 12:00:01 AM local: 1969-12-31 @ 06:00:01 PM (54 years 1 month 3 weeks 5 days 17 hours ago)
```

## License

MIT License, see <a href="LICENSE">LICENSE</a> file
