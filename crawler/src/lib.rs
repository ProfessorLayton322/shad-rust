#![forbid(unsafe_code)]

use linkify::{LinkFinder, LinkKind};
use tokio::sync::mpsc::{channel, Receiver, Sender};

use futures::StreamExt;
use std::collections::HashSet;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct Config {
    pub concurrent_requests: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            concurrent_requests: Some(1),
        }
    }
}

#[derive(Debug)]
pub struct Page {
    pub url: String,
    pub body: String,
}

pub struct Crawler {
    config: Config,
    worker: Option<tokio::task::JoinHandle<()>>,
}

async fn read_url(url: String) -> Page {
    let body: String = reqwest::get(&url).await.unwrap().text().await.unwrap();
    Page { url, body }
}

impl Crawler {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            worker: None,
        }
    }

    pub fn run(&mut self, site: String) -> Receiver<Page> {
        if self.worker.is_some() {
            panic!("Already running!");
        }
        let (sender, receiver) = channel::<Page>(self.config.concurrent_requests.unwrap());
        self.worker = Some(tokio::spawn(Self::worker(
            site,
            self.config.concurrent_requests.unwrap(),
            sender,
        )));
        receiver
    }

    async fn worker(site: String, concurrent_requests: usize, sender: Sender<Page>) {
        let mut registry: HashSet<String> = HashSet::new();
        registry.insert(site.clone());
        let mut finder = LinkFinder::new();
        finder.kinds(&[LinkKind::Url]);
        let mut futures = futures::prelude::stream::FuturesUnordered::new();
        futures.push(read_url(site));
        let mut queue: VecDeque<String> = VecDeque::new();
        while let Some(page) = futures.next().await {
            registry.insert(page.url.clone());
            for link in finder.links(&page.body).map(|l| l.as_str().to_string()) {
                if registry.contains(&link) {
                    continue;
                }
                queue.push_back(link.clone());
                registry.insert(link.clone());
            }
            for _ in 0..(concurrent_requests - futures.len()) {
                if let Some(url) = queue.pop_front() {
                    futures.push(read_url(url));
                }
            }
            sender.try_send(page).unwrap();
        }
    }
}
