extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate term;
extern crate rand;

use hyper::Client;
use hyper::client::Response;
use std::io::{BufReader, BufRead};
use std::sync::mpsc::*;
use std::thread;

include!(concat!(env!("OUT_DIR"), "/main.rs"));

#[derive(Debug)]
struct Record {
    namespace: String,
    pod: String,
    container: String,
    color: term::color::Color,
    text: String,
}

fn main() {
    let colors = vec![term::color::CYAN,
                      term::color::MAGENTA,
                      term::color::GREEN,
                      term::color::YELLOW,
                      term::color::BRIGHT_BLUE];
    let client = Client::new();
    let response = client.get("http://127.0.0.1:8001/api/v1/pods")
        .send()
        .unwrap();
    let pods = serde_json::from_reader::<Response, PodList>(response).unwrap();
    let (tx, rx) = channel();
    let mut t = term::stdout().unwrap();
    thread::spawn(move || {
        loop {
            if let Ok(Record { namespace, pod, container, color, text }) = rx.recv() {
                t.reset().unwrap();
                t.fg(color).unwrap();
                write!(t, "{}/{}/{}: ", namespace, pod, container).unwrap();
                t.reset().unwrap();
                writeln!(t, ": {}", text);
            }
        }
    });
    let mut rng = rand::thread_rng();
    for pod in pods.items {
        let containers = pod.spec.containers.iter().map(|c| c.name.clone()).collect::<Vec<_>>();
        let reader = BufReader::new(client.get(&format!("http://127.0.0.1:\
                           8001/api/v1/namespaces/{}/pods/{}/log?container={}&follow=true",
                          pod.metadata.namespace,
                          pod.metadata.name,
                          containers[0]))
            .send()
            .unwrap());
        let px = tx.clone();
        let color = rand::sample(&mut rng, colors.clone(), 1)[0];
        thread::spawn(move || {
            for l in reader.lines() {
                if let Ok(text) = l {
                    px.send(Record {
                            namespace: pod.metadata.namespace.clone(),
                            pod: pod.metadata.name.clone(),
                            container: containers[0].clone(),
                            color: color,
                            text: text,
                        })
                        .unwrap()
                }
            }
        });
    }
}
