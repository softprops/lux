use hyper::Client;
use hyper::client::Response;
use serde_json::StreamDeserializer;
use std::io::{BufRead, BufReader, Read};
use std::sync::mpsc::{Sender, channel};
use std::thread;
use super::Error;
use super::rand;
use super::serde_json;
use super::term;
use super::url;

include!(concat!(env!("OUT_DIR"), "/logs.rs"));

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

impl Logs {
    pub fn new(follow: bool, label: Option<String>, namespace: Option<String>) -> Logs {
        Logs {
            follow: follow,
            label: label,
            namespace: namespace,
        }
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
                   -> Vec<thread::JoinHandle<()>> {
            let mut tails = vec![];
            for container in pod.spec.containers {
                let pxc = px.clone();
                let this_namespace = pod.metadata.namespace.clone();
                let this_pod_name = pod.metadata.name.clone();
                let mut logs_endpoint = url::Url::parse(PROXY_HOST)
                    .unwrap()
                    .join(&format!("/api/v1/namespaces/{}/pods/{}/log",
                                   this_namespace,
                                   this_pod_name))
                    .unwrap();
                logs_endpoint.query_pairs_mut()
                    .extend_pairs(vec![("container", container.name.as_str()),
                                       ("follow", follow.to_string().as_str())]);
                let reader = BufReader::new(client.get(logs_endpoint).send().unwrap());

                tails.push(thread::spawn(move || {
                    for l in reader.lines() {
                        if let Ok(text) = l {
                            let _ = pxc.send(Record {
                                namespace: this_namespace.clone(),
                                pod: this_pod_name.clone(),
                                container: container.name.clone(),
                                color: color,
                                text: text,
                            });
                        }
                    }
                }))
            }
            tails
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
