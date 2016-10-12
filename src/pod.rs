use super::serde_json;
use serde_json::StreamDeserializer;
use std::io;
use super::Error;

include!(concat!(env!("OUT_DIR"), "/pod.rs"));

pub struct Pods {
    inner: Box<Iterator<Item = Pod>>,
}

impl Pods {
    pub fn new<Bytes>(follow: bool, bytes: Bytes) -> Result<Pods, Error>
        where Bytes: 'static + Iterator<Item = io::Result<u8>>
    {
        if follow {
            let s: StreamDeserializer<PodEvent, _> = StreamDeserializer::new(bytes);
            Ok(Pods { inner: Box::new(s.filter_map(|e| e.ok()).map(|e| e.object)) })
        } else {
            Ok(Pods {
                inner: Box::new(try!(serde_json::from_iter::<_, PodList>(bytes)).items.into_iter()),
            })
        }
    }
}

impl Iterator for Pods {
    type Item = Pod;
    fn next(&mut self) -> Option<Pod> {
        self.inner.next()
    }
}
