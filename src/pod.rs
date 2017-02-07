use std::io;
use super::Error;
use super::serde_json;

#[derive(Deserialize, Debug)]
pub struct PodEvent {
    pub object: Pod,
}

#[derive(Deserialize, Debug)]
pub struct PodList {
    pub items: Vec<Pod>,
}

#[derive(Deserialize, Debug)]
pub struct Pod {
    pub metadata: Metadata,
    pub spec: PodSpec,
}

#[derive(Deserialize, Debug)]
pub struct PodSpec {
    pub containers: Vec<Container>,
}

#[derive(Deserialize, Debug)]
pub struct Metadata {
    pub name: String,
    pub namespace: String,
}

#[derive(Deserialize, Debug)]
pub struct Container {
    pub name: String,
}

pub struct Pods {
    inner: Box<Iterator<Item = Pod>>,
}

impl Pods {
    pub fn new<Bytes>(follow: bool, bytes: Bytes) -> Result<Pods, Error>
        where Bytes: 'static + Iterator<Item = io::Result<u8>>
    {
        if follow {
            let s = serde_json::Deserializer::from_iter(bytes).into_iter::<PodEvent>();
            Ok(Pods { inner: Box::new(s.filter_map(|e| e.ok()).map(|e| e.object)) })
        } else {
            Ok(Pods {
                inner: Box::new(serde_json::from_iter::<_, PodList>(bytes)?.items.into_iter()),
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
