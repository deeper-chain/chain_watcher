use anyhow::Result;
use hex_literal::hex;
use tokio::process::Command;
use web3::{
    futures::{future, StreamExt},
    types::FilterBuilder,
};

#[tokio::main]
async fn main() -> Result<()> {
    let _ = env_logger::try_init();

    let output = Command::new("echo").arg("123").output().await?;
    //let output = Command::new("curl").arg("localhost/api/admin/getDeviceId").output().await?;
    let out_string = String::from_utf8_lossy(&output.stdout).to_string();
    println!("shell output {:?}", out_string);

    let transport = web3::transports::WebSocket::new("wss://mainnet-dev.deeper.network").await?;
    let web3 = web3::Web3::new(transport);

    let filter = FilterBuilder::default()
        //.address(vec![contract.address()])
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
        let sn = out_string.clone();
        tokio::spawn(async move {
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
