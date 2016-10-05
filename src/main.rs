extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate term;
extern crate rand;
extern crate url;

use hyper::Client;
use hyper::client::Response;
use std::io::{BufReader, BufRead};
use std::sync::mpsc::channel;
use std::thread;
use std::process;

include!(concat!(env!("OUT_DIR"), "/main.rs"));

#[derive(Debug)]
struct Record {
    namespace: String,
    pod: String,
    container: String,
    color: term::color::Color,
    text: String,
}

#[derive(Default)]
pub struct Options {
    follow: bool,
    label: Option<String>
}

pub struct Logs {
    options: Options,
}

#[derive(Debug)]
pub enum Error {
    Http,
    Parse,
    Url
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
    pub fn new(options: Options) -> Logs {
        Logs { options: options }
    }

    pub fn fetch(&self) -> Result<(), Error> {
        let colors = vec![term::color::CYAN,
                          term::color::MAGENTA,
                          term::color::GREEN,
                          term::color::YELLOW,
                          term::color::BRIGHT_BLUE];
        let client = Client::new();
        let mut pods_endpoint = try!(url::Url::parse("http://127.0.0.1:8001/api/v1/pods"));
        if let Some(ref label) = self.options.label {
            pods_endpoint.query_pairs_mut().append_pair("labelSelector", label);
        }
        let response = try!(client.get(pods_endpoint).send());
        let pods = try!(serde_json::from_reader::<Response, PodList>(response));
        let (tx, rx) = channel();
        let mut t = term::stdout().unwrap();
        thread::spawn(move || {
            loop {
                if let Ok(Record { namespace, pod, container, color, text }) = rx.recv() {
                    t.reset().unwrap();
                    t.fg(color).unwrap();
                    write!(t, "{}/{}/{}: ", namespace, pod, container).unwrap();
                    t.reset().unwrap();
                    writeln!(t, ": {}", text).unwrap();
                }
            }
        });
        let mut rng = rand::thread_rng();
        for pod in pods.items {
            let containers = pod.spec.containers.iter().map(|c| c.name.clone()).collect::<Vec<_>>();
            let mut logs_endpoint = try!(url::Url::parse("http://127.0.0.1:8001/api/v1/namespaces/"))
                .join(format!("{}/", pod.metadata.namespace).as_ref()).unwrap()
                .join("pods/").unwrap()
                .join(format!("{}/log", pod.metadata.name).as_ref()).unwrap();
            logs_endpoint.query_pairs_mut().extend_pairs(vec![
                ("container", containers[0].as_str()),
                ("follow", self.options.follow.to_string().as_str())]);
            println!("url {}", logs_endpoint.as_str());
            let reader = BufReader::new(try!(client.get(logs_endpoint).send()));
            let px = tx.clone();
            let color = rand::sample(&mut rng, colors.clone(), 1)[0];
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
            });
        }
        Ok(())
    }
}

fn main() {
    if let Err(e) = Logs::new(Default::default()).fetch() {
        println!("error fetching logs: {:?}", e);
        process::exit(1);
    }
}
