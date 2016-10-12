use hyper::Client;
use std::io::{BufRead, BufReader, Read};
use std::sync::mpsc::channel;
use std::thread;
use super::Error;
use super::color;
use super::pod::Pods;
use super::term;
use super::url;

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
        let mut colors = color::Wheel::new();
        let (tx, rx) = channel();
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

        for pod in try!(Pods::new(self.follow, response.bytes())) {
            let color = colors.next().unwrap();
            for container in pod.spec.containers {
                let pxc = tx.clone();
                let namespace = pod.metadata.namespace.clone();
                let pod_name = pod.metadata.name.clone();
                let mut logs_endpoint = url::Url::parse(PROXY_HOST)
                    .unwrap()
                    .join(&format!("/api/v1/namespaces/{}/pods/{}/log", namespace, pod_name))
                    .unwrap();
                logs_endpoint.query_pairs_mut()
                    .extend_pairs(vec![("container", container.name.as_str()),
                                       ("follow", self.follow.to_string().as_str())]);
                let reader = BufReader::new(client.get(logs_endpoint).send().unwrap());

                thread::spawn(move || {
                    for l in reader.lines() {
                        if let Ok(text) = l {
                            let _ = pxc.send(Record {
                                namespace: namespace.clone(),
                                pod: pod_name.clone(),
                                container: container.name.clone(),
                                color: color,
                                text: text,
                            });
                        }
                    }
                });
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
