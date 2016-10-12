#[macro_use]
extern crate clap;
extern crate lux;

use clap::App;
use lux::Logs;
use std::process;

fn main() {
    let args = App::new("lux")
        .about("a kubernetes log multiplexor")
        .args_from_usage("-l, --label=[LABEL] 'Label selector filter'
             -f, --follow \
                          'Follow the logs as they are available'
             -n, \
                          --namespace=[NAMESPACE] 'Filter logs to a target namespace'
             \
                          -t, --timestamps 'Print record timestamps'
             --tail=[N] \
                          'Number of recent logs to display'
             --since=[SECONDS] \
                          'Prints records since this a given number of seconds. Only one of \
                          since-time / since may be used.'
              \
                          --since-time=[RFC3339_TIMESTAMP] 'Prints records since the given \
                          timestamp Only one of since-time / since may be used.'
        ")
        .get_matches();

    let logs = Logs::new(args.occurrences_of("follow") > 0,
                         args.value_of("label").map(|s| s.to_owned()),
                         args.value_of("namespace").map(|s| s.to_owned()),
                         args.occurrences_of("previous") > 0,
                         args.value_of("since")
                             .map(|s| s.parse::<usize>().expect("since must be an int")),
                         args.value_of("since-time").map(|s| s.to_owned()),
                         args.value_of("tail")
                             .map(|s| s.parse::<usize>().expect("tail must be an int")),
                         args.occurrences_of("timestamps") > 0);
    if let Err(e) = logs.fetch() {
        println!("error fetching logs: {}", e);
        process::exit(1);
    }
}
