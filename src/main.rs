mod worker;

use crate::worker::{Worker, WorkerMsg};
use anyhow::Result;
use clap::Parser;
use hex::FromHex;
use lru::LruCache;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::{mpsc, Mutex};
use web3::{
    ethabi::{self, param_type::ParamType, Token},
    types::{BlockNumber, FilterBuilder},
};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[clap(
        short = 'c',
        long,
        default_value = "https://mainnet-dev.deeper.network/rpc"
    )]
    chain: String,

    #[clap(short = 'u', long, required = false)]
    url: Option<String>,

    #[clap(
        short = 't',
        long,
        default_value = "ff68b5ae1c6eef082af114f218b96313f8eaa0e0ccbf5a4d2795eab86b5fdec4"
    )]
    topic: String,

    #[clap(short = 'w', long, default_value_t = 10)]
    worker_count: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = env_logger::try_init();

    let args = Args::parse();
    println!("{:?} {}!", args.url, args.chain);

    //let output = Command::new("echo").arg("123456").output().await?;
    let out_string = {
        let output = Command::new("curl")
            .arg("localhost/api/admin/getDeviceId")
            .output()
            .await?;

        if output.stdout.is_empty() {
            let buffer = std::fs::read_to_string("/root/config/public_keys.json")?;
            let obj = json::parse(&buffer)?;
            obj["pubkey"].as_str().unwrap_or_default().to_string()
        } else {
            String::from_utf8_lossy(&output.stdout).to_string()
        }
    };

    println!("shell output {:?}", out_string);
    //let (tx, _rx) = mpsc::unbounded_channel();
    let _ = tokio::spawn(async move {
        println!("inspect_chain_event begin");
        let _ = inspect_chain_event(
            args.chain,
            args.url.unwrap_or_default(),
            args.topic,
            out_string,
            args.worker_count,
        )
        .await;
    })
    .await;
    Ok(())
}

pub async fn inspect_chain_event(
    chain: String,
    _url: String,
    topic: String,
    out_string: String,
    worker_count: usize,
    //rx: mpsc::UnboundedReceiver<Message>,
) -> Result<()> {
    //let url= "http://localhost:9933";

    let topic_hash = <[u8; 32]>::from_hex(topic)?;
    let transport = web3::transports::Http::new(&chain.clone())?;
    let mut web3 = web3::Web3::new(transport);
    let mut base_number = web3.eth().block_number().await?;
    println!("init block number {:?}", base_number);
    let mut dst_number = base_number;

    let images = Arc::new(Mutex::new(LruCache::new(10)));
    let mut txs = Vec::new();
    let mut idx = 0;

    for _ in 0..worker_count {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut worker = Worker::new(out_string.clone(), images.clone(), rx);
        txs.push(tx);

        let _ = tokio::spawn(async move {
            let _ = worker.run().await;
        });
    }

    loop {
        while base_number >= dst_number {
            tokio::time::sleep(std::time::Duration::new(8, 0)).await;
            let maybe_num = web3.eth().block_number().await;
            if maybe_num.is_err() {
                break;
            } else {
                dst_number = maybe_num.unwrap();
                println!("now dst block number {:?}", dst_number);
            }
        }

        let filter = FilterBuilder::default()
            .from_block(BlockNumber::Number(base_number))
            .to_block(BlockNumber::Number(dst_number))
            .topics(Some(vec![topic_hash.into()]), None, None, None)
            .build();

        let logs = web3.eth().logs(filter).await;
        if let Ok(logs) = logs {
            for log in logs {
                println!("got log: {:?}", log);
                let parse_res =
                    ethabi::decode(&[ParamType::String, ParamType::String], &log.data.0);
                let parse_res = parse_res.unwrap_or_default();

                let strs: Vec<String> = parse_res
                    .into_iter()
                    .filter(|t| t.type_check(&ParamType::String))
                    .map(|t| {
                        if let Token::String(res) = t {
                            res
                        } else {
                            "".to_string()
                        }
                    })
                    .collect();
                if strs.len() != 2 {
                    continue;
                }
                let _ = txs[idx].send(WorkerMsg::DockerInfo(strs[0].clone(), strs[1].clone()));
                idx = (idx + 1) % txs.len();
            }

            base_number = dst_number + 1;
        } else {
            let chain = chain.clone();
            loop {
                let reconn = web3::transports::Http::new(&chain.clone());
                println!("reconn res {:?}", reconn);
                if reconn.is_err() {
                    tokio::time::sleep(std::time::Duration::new(3, 0)).await;
                } else {
                    web3 = web3::Web3::new(reconn.unwrap());
                    break;
                }
            }
        }
    }
}
