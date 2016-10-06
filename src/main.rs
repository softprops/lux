#[macro_use]
extern crate clap;
extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate term;
extern crate rand;
extern crate url;

use serde_json::StreamDeserializer;
use hyper::Client;
use hyper::client::Response;
use std::io::{BufReader, BufRead};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::process;
use std::io::Read;

include!(concat!(env!("OUT_DIR"), "/main.rs"));

const PROXY_HOST: &'static str = "http://127.0.0.1:8001";

#[derive(Debug)]
struct Record {
    namespace: String,
    pod: String,
    container: String,
    color: term::color::Color,
    text: String,
}

#[derive(Default, Debug)]
pub struct Logs {
    follow: bool,
    label: Option<String>,
    namespace: Option<String>,
}

#[derive(Debug)]
pub enum Error {
    Http,
    Parse,
    Url,
}

impl From<url::ParseError> for Error {
    fn from(_: url::ParseError) -> Error {
        Error::Url
    }
}

impl From<hyper::Error> for Error {
    fn from(_: hyper::Error) -> Error {
        Error::Http
    }
}

impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Error {
        Error::Parse
    }
}

impl Logs {
    pub fn new() -> Logs {
        Logs { follow: false, ..Default::default() }
    }

    pub fn fetch(&self) -> Result<(), Error> {
        let colors = vec![term::color::CYAN,
                          term::color::MAGENTA,
                          term::color::GREEN,
                          term::color::YELLOW,
                          term::color::BRIGHT_BLUE];
        let mut rng = rand::thread_rng();
        let client = Client::new();
        let mut pods_endpoint = match self.namespace {
            Some(ref ns) => {
                try!(try!(url::Url::parse(PROXY_HOST))
                    .join(&format!("/api/v1/namespaces/{}/pods", ns)))
            }
            _ => try!(try!(url::Url::parse(PROXY_HOST)).join("/api/v1/pods")),

        };
        if let Some(ref label) = self.label {
            pods_endpoint.query_pairs_mut().append_pair("labelSelector", label);
        }
        if self.follow {
            pods_endpoint.query_pairs_mut().append_pair("watch", true.to_string().as_str());
        }
        let response = try!(client.get(pods_endpoint).send());
        let (tx, rx) = channel();
        // recv records from the channel
        let mut t = term::stdout().unwrap();
        thread::spawn(move || {
            loop {
                if let Ok(Record { namespace, pod, container, color, text }) = rx.recv() {
                    t.reset().unwrap();
                    t.fg(color).unwrap();
                    write!(t, "{}/{}/{}: ", namespace, pod, container).unwrap();
                    t.reset().unwrap();
                    writeln!(t, " {}", text).unwrap();
                }
            }
        });

        fn podlogs(client: &Client,
                   follow: bool,
                   pod: Pod,
                   px: Sender<Record>,
                   color: term::color::Color)
                   -> thread::JoinHandle<()> {
            let containers = pod.spec.containers.iter().map(|c| c.name.clone()).collect::<Vec<_>>();
            let mut logs_endpoint = url::Url::parse(PROXY_HOST)
                .unwrap()
                .join(&format!("/api/v1/namespaces/{}/pods/{}/log",
                               pod.metadata.namespace,
                               pod.metadata.name))
                .unwrap();
            logs_endpoint.query_pairs_mut()
                .extend_pairs(vec![("container", containers[0].as_str()),
                                   ("follow", follow.to_string().as_str())]);
            let reader = BufReader::new(client.get(logs_endpoint).send().unwrap());

            thread::spawn(move || {
                for l in reader.lines() {
                    if let Ok(text) = l {
                        let _ = px.send(Record {
                            namespace: pod.metadata.namespace.clone(),
                            pod: pod.metadata.name.clone(),
                            container: containers[0].clone(),
                            color: color,
                            text: text,
                        });
                    }
                }
            })
        }

        if self.follow {
            let stream: StreamDeserializer<PodEvent, _> = StreamDeserializer::new(response.bytes());
            for e in stream {
                match e {
                    Ok(event) => {
                        let _ = podlogs(&client,
                                        self.follow,
                                        event.object,
                                        tx.clone(),
                                        rand::sample(&mut rng, colors.clone(), 1)[0]);
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        } else {
            let pods = try!(serde_json::from_reader::<Response, PodList>(response));
            for pod in pods.items {
                let _ = podlogs(&client,
                                self.follow,
                                pod,
                                tx.clone(),
                                rand::sample(&mut rng, colors.clone(), 1)[0]);
            }
        }

        Ok(())
    }
}

fn main() {
    let args = clap::App::new("lux")
        .about("a kubernetes log multiplexor")
        .args_from_usage("-l, --label=[LABEL] 'label selector filter'
             -f, --follow \
                          'follow the logs as they are available'
             -n, \
                          --namespace=[NAMESPACE] 'filter logs to a target namespace'")
        .get_matches();
    let logs = Logs {
        follow: args.occurrences_of("follow") > 0,
        label: args.value_of("label").map(|s| s.to_owned()),
        namespace: args.value_of("namespace").map(|s| s.to_owned()),
    };
    if let Err(e) = logs.fetch() {
        println!("error fetching logs: {:?}", e);
        process::exit(1);
    }
}
