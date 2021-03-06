extern crate askama;
extern crate rsass;
extern crate time;

use rsass::{compile_scss_file, OutputStyle};
use time::now_utc;

use std::process::Command as Cmd;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::env;

const VERSION_FILE_NAME: &str = "fht2p.txt";
const CSS_PATH: &str = "templates/fht2p.css";
const CSS_FILE_NAME: &str = "fht2p.css";

fn main() {
    askama::rerun_if_templates_changed();   

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    css(&out_dir).unwrap();
    version(&out_dir).unwrap();
}

// use sass to compress css file
fn css(out_dir: &PathBuf) -> io::Result<()> {
    let css = compile_scss_file(Path::new(CSS_PATH), OutputStyle::Compressed).unwrap();
    let out_path = out_dir.join(CSS_FILE_NAME);
    File::create(&out_path).and_then(|mut f| f.write_all(css.as_slice()))
}

// include!(concat!(env!("OUT_DIR"), "/fht2p.txt"));
fn version(out_dir: &PathBuf) -> io::Result<()> {
    let out_path = out_dir.join(VERSION_FILE_NAME);
    File::create(&out_path).and_then(|mut f| f.write_all(fun().as_bytes()))
}

fn fun() -> String {
    let rustc = rustc_version()
        .map(|s| format!(" rustc{}", s.split(' ').nth(1).unwrap()))
        .unwrap_or_default();
    let git = commit_hash()
        .map(|s| (&s[0..8]).to_string())
        .and_then(|s| branch_name().map(|b| format!("{}@{}{} ", s, b, rustc)))
        .unwrap_or_default();

    let version = format!("{} ({}{})", env!("CARGO_PKG_VERSION"), git, date_time());
    format!("pub const VERSION: &str = \"{}\";", version)
}

// date --help
fn date_time() -> String {
    now_utc()
    // .strftime("%Y-%m-%d/%H:%M:%SUTC")
    .strftime("%Y-%m-%dUTC")
    .map(|dt| dt.to_string())
    .unwrap_or_default()
}

fn commit_hash() -> io::Result<String> {
    Cmd::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .map(|o| decode(&o.stdout))
        .map(|s| s.trim().to_string())
}

fn branch_name() -> io::Result<String> {
    Cmd::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map(|o| decode(&o.stdout).trim().to_string())
}

fn rustc_version() -> io::Result<String> {
    Cmd::new("rustc")
        .arg("--version")
        .output()
        .map(|o| decode(&o.stdout).trim().to_string())
}

fn decode(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned().to_owned()
}
