use anyhow::Result;
use lru::LruCache;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::{mpsc::UnboundedReceiver, Mutex};
use tokio::time::timeout;

pub enum WorkerMsg {
    //url ,params
    DockerInfo(String, String),
}

pub struct Worker {
    sn: String,
    images: Arc<Mutex<LruCache<String, bool>>>,
    rx: UnboundedReceiver<WorkerMsg>,
}

pub async fn send_request(sn: String) -> Result<()> {
    let url = format!("http://81.68.122.162:8000?sn={}", sn);
    let res = reqwest::get(url.clone()).await?.text().await?;
    println!("url {} res {:?}", url, res);
    Ok(())
}

impl Worker {
    pub fn new(
        sn: String,
        images: Arc<Mutex<LruCache<String, bool>>>,
        rx: UnboundedReceiver<WorkerMsg>,
    ) -> Self {
        Self { sn, images, rx }
    }

    pub async fn run(&mut self) -> Result<()> {
        // let nurl = url.clone();
        // let names: Vec<&str> = nurl.rsplit_terminator('/').collect();
        loop {
            let sn = self.sn.clone();
            if let Some(WorkerMsg::DockerInfo(url, params)) = self.rx.recv().await {
                println!("args url {} ", url);
                let mut dropped = String::new();
                let time_duration = std::time::Duration::new(3600, 0);
                let pull_required = self.images.lock().await.get(&url).is_none();
                if pull_required {
                    let pull_res = timeout(
                        time_duration,
                        Command::new("docker").arg("pull").arg(url.clone()).output(),
                    )
                    .await;
                    if pull_res.is_err() {
                        println!("docker pull {} timeout", url);
                        continue;
                    }

                    let pull_res = pull_res.unwrap();
                    println!("docker pull res {:?}", pull_res);
                    if pull_res.is_ok() && pull_res.unwrap().status.success() {
                        if let Some((old, _)) = self.images.lock().await.push(url.clone(), true) {
                            if old != url {
                                dropped = old;
                            }
                        }
                    } else {
                        continue;
                    }
                }

                let output = Command::new("docker")
                    .arg("run")
                    .arg("--rm")
                    .arg(url.clone())
                    .arg(params)
                    .output()
                    .await?;
                println!("docker run res {:?}", output);

                let _ = tokio::spawn(async move {
                    let _ = send_request(sn.clone()).await;
                })
                .await;

                if !dropped.is_empty() {
                    println!("docker image droped {:?}", dropped);
                    let _ = Command::new("docker").arg("rmi").arg(url).output().await;
                }
            } else {
                break;
            }
        }
        Ok(())
    }
}
