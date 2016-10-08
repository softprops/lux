#[macro_use]
extern crate clap;
extern crate lux;

use clap::App;
use lux::Logs;
use std::process;

fn main() {
    let args = App::new("lux")
        .about("a kubernetes log multiplexor")
        .args_from_usage("-l, --label=[LABEL] 'label selector filter'
             -f, --follow \
                          'follow the logs as they are available'
             -n, \
                          --namespace=[NAMESPACE] 'filter logs to a target namespace'")
        .get_matches();
    let logs = Logs::new(args.occurrences_of("follow") > 0,
                         args.value_of("label").map(|s| s.to_owned()),
                         args.value_of("namespace").map(|s| s.to_owned()));
    if let Err(e) = logs.fetch() {
        println!("error fetching logs: {}", e);
        process::exit(1);
    }
}
