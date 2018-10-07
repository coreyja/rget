#[macro_use]
extern crate serde_derive;
extern crate docopt;

use docopt::Docopt;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const USAGE: &'static str = "
Rget.

Usage:
  rget <url>
  rget (-h | --help)
  rget --version

Options:
  -h --help     Show this screen.
  --version     Show version.
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_url: String,
    flag_version: bool
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.deserialize())
                            .unwrap_or_else(|e| e.exit());

		if args.flag_version {
			println!("Version: {}", VERSION);
		} else {
			println!("Hello, world!");
			println!("{:?}", args);
		}
}
