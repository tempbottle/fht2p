/*!
# A HTTP Server for Static File written with Rust.

## Support Unix-like and windows 7+.

## Snapshot

![snapshot.png](https://raw.githubusercontent.com/biluohc/fht2p/master/config/assets/snapshot.png)

## Usage  
```sh
    cargo install --git https://github.com/biluohc/fht2p fht2p -f

    # running fht2p --help(-h) to get help.

    fht2p -h
```
### Or
```sh
    git clone https://github.com/biluohc/fht2p
    # cargo  install --path fht2p/ fht2p -f
    
    cd fht2p 
    cargo build --release

    ./target/release/fht2p --help
```

## Help
```sh
fht2p 0.8.2 (f9dbc530@master rustc1.27.0-nightly 2018-04-18UTC)
A HTTP Server for Static File written with Rust
Wspsxing <biluohc@qq.com>
Github: https://github.com/biluohc/fht2p

USAGE:
   fht2p [options] [<PATH>...]

OPTIONS:
   -h, --help                               Show the help message
   -V, --version                            Show the version message
   -r, --redirect-html                      Redirect dir to `index.html/htm`, if it exists
   -m, --magic-limit <byte>[10485760]       The limit for detect file ContenType(use 0 to close)
   -k, --keep-alive                         Close HTTP keep alive
   -c, --config <config>(optional)          Sets a custom config file
   -C, --config-print                       Print the default config file
   -s, --cache-secs <secs>[60]              Sets cache secs(use 0 to close)
   -f, --follow-links                       Whether follow links(default follow)
   -i, --ip <ip>[0.0.0.0]                   Sets listenning ip
   -p, --port <port>[8080]                  Sets listenning port

ARGS:
   <PATH>["./"]     Sets the paths to share
```
*/
#![allow(unknown_lints, clone_on_ref_ptr, boxed_local)]
#[macro_use]
extern crate log;
extern crate mxo_env_logger;
use mxo_env_logger::*;
extern crate app;
extern crate bytes;
extern crate chrono;
extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
#[macro_use]
extern crate hyper_fs;
#[macro_use]
extern crate lazy_static;
extern crate mime_guess;
#[macro_use]
extern crate serde_derive;
extern crate tokio_core;
extern crate toml;
extern crate url;
#[macro_use]
extern crate askama;

#[macro_use(signalfn, ctrlcfn)]
extern crate signalfn;
extern crate systemstat;
use signalfn::register_ctrlcfn;

pub(crate) mod consts;
pub(crate) mod content_type;
pub(crate) mod exception;
pub(crate) mod server;
pub(crate) mod router;
pub(crate) mod views;
pub(crate) mod index;
pub(crate) mod tools;
pub(crate) mod args;
pub(crate) mod stat;

use std::process::exit;

fn callback() {
    exit(0)
}

ctrlcfn!(ctrlc_exit, callback);

fn main() {
    init().expect("Init log failed");

    let config = args::parse();
    debug!("{:?}", config);

    register_ctrlcfn(ctrlc_exit)
        .map_err(|e| error!("Register CtrlC Signal failed: {:?}", e))
        .ok();

    if let Err(e) = server::run(config) {
        error!("{}", e.description());
        exit(1);
    }
}
