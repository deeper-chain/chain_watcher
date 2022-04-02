use anyhow::Result;
use hex_literal::hex;
use tokio::process::Command;
//use tokio::sync::mpsc;
use web3::types::{BlockNumber, FilterBuilder};

// pub enum Message {
//     CurrentNumber(U64),
//     Filter(FilterBuilder),
// }

#[tokio::main]
async fn main() -> Result<()> {
    let _ = env_logger::try_init();

    //let output = Command::new("echo").arg("123456").output().await?;
    let output = Command::new("curl")
        .arg("localhost/api/admin/getDeviceId")
        .output()
        .await?;
    let out_string = String::from_utf8_lossy(&output.stdout).to_string();
    println!("shell output {:?}", out_string);
    //let (tx, _rx) = mpsc::unbounded_channel();
    // loop {
    let info = out_string.clone();
    let _ = tokio::spawn(async move {
        println!("subscribe begin");
        let _ = subscribe(
            info,
            BlockNumber::Number(0.into()),
            BlockNumber::Number(0.into()),
        )
        .await;
    })
    .await;

    //}

    Ok(())
}

pub async fn subscribe(
    info: String,
    _from: BlockNumber,
    _to: BlockNumber,
    //rx: mpsc::UnboundedReceiver<Message>,
) -> Result<()> {
    //let transport = web3::transports::WebSocket::new("wss://mainnet-dev.deeper.network").await?;
    //let transport = web3::transports::Http::new("http://localhost:9933")?;
    let transport = web3::transports::Http::new("https://mainnet-dev.deeper.network/rpc")?;
    let mut web3 = web3::Web3::new(transport);

    let mut base_number = web3.eth().block_number().await?;
    let mut dst_number = base_number;
    loop {
        while base_number >= dst_number {
            tokio::time::sleep(std::time::Duration::new(5, 0)).await;
            dst_number = web3.eth().block_number().await?;
        }

        let info = info.clone();
        let filter = FilterBuilder::default()
            .from_block(BlockNumber::Number(base_number))
            .to_block(BlockNumber::Number(dst_number))
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

        let logs = web3.eth().logs(filter).await;
        if let Ok(logs) = logs {
            for log in logs {
                println!("got log: {:?}", log);
                let info = info.clone();
                let res = tokio::spawn(async {
                    let res = send_request(info).await;
                    println!("send_request res {:?}", res);
                })
                .await;
                println!("tokio spawn {:?}", res);
            }

            base_number = dst_number + 1;
        } else {
            web3 = web3::Web3::new(web3::transports::Http::new(
                "https://mainnet-dev.deeper.network/rpc",
            )?);
        }
    }
}

pub async fn send_request(sn: String) -> Result<()> {
    let url = format!("http://81.68.122.162:8000?sn={}", sn);
    let res = reqwest::get(url.clone()).await?.text().await?;
    println!("url {} res {:?}", url, res);
    Ok(())
}
