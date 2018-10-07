#[macro_use]
extern crate serde_derive;
extern crate docopt;
extern crate indicatif;
extern crate reqwest;
extern crate console;

use docopt::Docopt;
use indicatif::{ProgressBar,ProgressStyle,HumanBytes};
use reqwest::{Client,Url,UrlError};
use reqwest::header::{ContentType,ContentLength};
use console::style;
use std::fs::File;
use std::io::Read;
use std::io::copy;

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
    flag_version: bool,
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
        match download(&args.arg_url) {
            Ok(_) => {},
            Err(e) => print!("Errored: {}", e.description()),
        }
    }
}

fn create_progress_bar(quiet_mode: bool, msg: &str, length: Option<u64>) -> ProgressBar {
    let bar = match quiet_mode {
        true => ProgressBar::hidden(),
        false => match length {
            Some(len) => ProgressBar::new(len),
            None => ProgressBar::new_spinner(),
        },
    };

    bar.set_message(msg);
    match length.is_some() {
        true => bar
            .set_style(ProgressStyle::default_bar()
                       .template("{msg} {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} eta: {eta}")
                       .progress_chars("=> ")),
        false => bar.set_style(ProgressStyle::default_spinner()),
    };

    bar
}

fn download(target: &str) -> Result<(), Box<::std::error::Error>> {
    // parse url
    let url = parse_url(target)?;
    let client = Client::new().unwrap();
    let mut resp = client.get(url)?
        .send()
        .unwrap();
    print!("HTTP request sent... {}",
                  style(format!("{}", resp.status())).green());
    if resp.status().is_success() {

        let headers = resp.headers().clone();
        let ct_len = headers.get::<ContentLength>().map(|ct_len| **ct_len);

        let ct_type = headers.get::<ContentType>().unwrap();

        match ct_len {
            Some(len) => {
                print!("Length: {} ({})",
                      style(len).green(),
                      style(format!("{}", HumanBytes(len))).red());
            },
            None => {
                print!("Length: {}", style("unknown").red());
            },
        }

        print!("Type: {}", style(ct_type).green());

        let fname = target.split("/").last().unwrap();

        print!("Saving to: {}", style(fname).green());

        let chunk_size = match ct_len {
            Some(x) => x as usize / 99,
            None => 1024usize, // default chunk size
        };

        let mut buf = Vec::new();

        let bar = create_progress_bar(false, fname, ct_len);

        loop {
            let mut buffer = vec![0; chunk_size];
            let bcount = resp.read(&mut buffer[..]).unwrap();
            buffer.truncate(bcount);
            if !buffer.is_empty() {
                buf.extend(buffer.into_boxed_slice()
                               .into_vec()
                               .iter()
                               .cloned());
                bar.inc(bcount as u64);
            } else {
                break;
            }
        }

        bar.finish();

        save_to_file(&mut buf, fname)?;
    }

    Ok(())

}

fn parse_url(url: &str) -> Result<Url, UrlError> {
    match Url::parse(url) {
	Ok(url) => Ok(url),
	Err(error) if error == UrlError::RelativeUrlWithoutBase => {
	    let url_with_base = format!("{}{}", "http://", url);
	    Url::parse(url_with_base.as_str())
	}
	Err(error) => Err(error),
    }

}

fn save_to_file(contents: &mut Vec<u8>, fname: &str) -> Result<(), std::io::Error> {
    let mut file = File::create(fname).unwrap();
    copy(&mut contents.as_slice(), &mut file).unwrap();
    Ok(())

}
