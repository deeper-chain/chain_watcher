use anyhow::Result;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::{mpsc::UnboundedReceiver, Mutex};

pub enum WorkerMsg {
    //url ,params
    DockerInfo(String, String),
}

pub struct Worker {
    sn: String,
    images: Arc<Mutex<HashSet<String>>>,
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
        images: Arc<Mutex<HashSet<String>>>,
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

                let pull_required = !self.images.lock().await.contains(&url);
                if pull_required {
                    let output = Command::new("docker")
                        .arg("pull")
                        .arg(url.clone())
                        .output()
                        .await?;
                    println!("docker pull res {:?}", output);
                    if output.status.success() {
                        println!("docker pull success");
                        self.images.lock().await.insert(url.clone());
                    }
                }

                let output = Command::new("docker")
                    .arg("run")
                    .arg("--rm")
                    .arg(url)
                    .arg(params)
                    .output()
                    .await?;
                println!("docker run res {:?}", output);

                let _ = tokio::spawn(async move {
                    let _ = send_request(sn.clone()).await;
                })
                .await;
            } else {
                break;
            }
        }
        Ok(())
    }
}
