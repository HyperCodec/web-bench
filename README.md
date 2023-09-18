# web-bench
A small tool to benchmark webservers

### Installation
Simply install Rust if you haven't already, and do `cargo install web-bench`

### Usage
Once installed, the command for this application is `wb`.

This command has 3 parameters:
- secs (time) - default: 30
- thread_count - default: 1
- url - required

Run `wb --help` for more information.

You can also set the `RUST_LOG` env variable for debugging purposes.

### Examples
- `wb -s 120 -t 1000 -u https://google.com`
- `wb --url https://discord.com`
- `wb --thread-count 10 -u https://example.org`
- `wb --secs 60 -u https://github.com`

### License
This project is licensed under the `MIT` license