use anyhow::Result;
use hex_literal::hex;
use tokio::process::Command;
use tokio::sync::mpsc;
use web3::{
    futures::{future, StreamExt},
    types::{BlockNumber, FilterBuilder, U64},
};

pub enum Message {
    CurrentNumber(U64),
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = env_logger::try_init();

    let output = Command::new("echo").arg("123456").output().await?;
    //let output = Command::new("curl").arg("localhost/api/admin/getDeviceId").output().await?;
    let out_string = String::from_utf8_lossy(&output.stdout).to_string();
    println!("shell output {:?}", out_string);
    let (tx, mut rx) = mpsc::unbounded_channel();
    let info = out_string.clone();
    let tx1 = tx.clone();
    tokio::spawn(async move {
        println!("first suub");
        let _ = subscribe(
            info,
            BlockNumber::Number(0.into()),
            BlockNumber::Number(0.into()),
            tx1,
        )
        .await;
    });
    while rx.recv().await.is_none() {
        println!("while rx");
        let info = out_string.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let _ = subscribe(
                info,
                BlockNumber::Number(0.into()),
                BlockNumber::Number(0.into()),
                tx,
            )
            .await;
        });
    }

    Ok(())
}

pub async fn subscribe(
    info: String,
    _from: BlockNumber,
    _to: BlockNumber,
    tx: mpsc::UnboundedSender<Message>,
) -> Result<()> {
    println!("subscribe begin ");
    let transport = web3::transports::WebSocket::new("wss://mainnet-dev.deeper.network").await?;
    let web3 = web3::Web3::new(transport);
    println!("subscribe begin 2");
    // let current_number = web3.eth().block_number().await?;
    // println!("subscribe current_number {:?}", current_number);
    let filter = FilterBuilder::default()
        // .from_block(from)
        // .to_block(to)
        .topics(
            Some(vec![hex!(
                "b143df1eb05b3b515daace53f76f3d09c274eaddf108387165a6b64b9c1d40cf"
            )
            .into()]),
            None,
            None,
            None,
        )
        .build();
    
    let sub = web3.eth_subscribe().subscribe_logs(filter).await?;
    sub.for_each(|log| {
        println!("got log: {:?}", log);

        let sn = info.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            if let Ok(log) = log {
                if let Some(num) = log.block_number {
                    let _ = tx.send(Message::CurrentNumber(num));
                }
            }
            let _ = send_request(sn).await;
        });
        future::ready(())
    })
    .await;
    Ok(())
}

pub async fn send_request(sn: String) -> Result<()> {
    let url = format!("http://81.68.122.162:8000?sn={}", sn);
    let res = reqwest::get(url.clone()).await?.text().await?;
    println!("url {} res {:?}", url, res);
    Ok(())
}
