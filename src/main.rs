use anyhow::Result;
use tokio::process::Command;
//use tokio::sync::mpsc;
use clap::Parser;
use hex::FromHex;
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = env_logger::try_init();

    let args = Args::parse();
    println!("{:?} {}!", args.url, args.chain);

    //let output = Command::new("echo").arg("123456").output().await?;
    let output = Command::new("curl")
        .arg("localhost/api/admin/getDeviceId")
        .output()
        .await?;
    let out_string = String::from_utf8_lossy(&output.stdout).to_string();
    println!("shell output {:?}", out_string);
    //let (tx, _rx) = mpsc::unbounded_channel();
    let _ = tokio::spawn(async move {
        println!("subscribe begin");
        let _ = subscribe(
            args.chain,
            args.url.unwrap_or_default(),
            args.topic,
            out_string,
            BlockNumber::Number(0.into()),
            BlockNumber::Number(0.into()),
        )
        .await;
    })
    .await;
    Ok(())
}

pub async fn subscribe(
    chain: String,
    _url: String,
    topic: String,
    _out_string: String,
    _from: BlockNumber,
    _to: BlockNumber,
    //rx: mpsc::UnboundedReceiver<Message>,
) -> Result<()> {
    //let url= "http://localhost:9933";

    let topic_hash = <[u8; 32]>::from_hex(topic)?;
    let transport = web3::transports::Http::new(&chain.clone())?;
    let mut web3 = web3::Web3::new(transport);
    let mut base_number = web3.eth().block_number().await?;
    println!("init block number {:?}", base_number);
    let mut dst_number = base_number;
    loop {
        while base_number >= dst_number {
            tokio::time::sleep(std::time::Duration::new(5, 0)).await;
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
                    .map(|t| match t {
                        Token::String(res) => res,
                        _ => "".to_string(),
                    })
                    .filter(|s| !s.is_empty())
                    .collect();
                if strs.len() != 2 {
                    continue;
                }

                //let info = info.clone();
                let _ = tokio::spawn(async move {
                    let _ = run_docker(strs[0].clone(), strs[1].clone()).await;
                })
                .await;
            }

            base_number = dst_number + 1;
        } else {
            let chain = chain.clone();
            loop {
                let reconn = web3::transports::Http::new(&chain.clone());
                println!("reconn res {:?}", reconn);
                if reconn.is_err() {
                    tokio::time::sleep(std::time::Duration::new(1, 0)).await;
                } else {
                    web3 = web3::Web3::new(reconn.unwrap());
                    break;
                }
            }
        }
    }
}

pub async fn run_docker(url: String, params: String) -> Result<()> {
    // if url.is_empty() {
    //     return Err(anyhow::Error());
    // }
    let nurl = url.clone();
    let names: Vec<&str> = nurl.rsplit_terminator('/').collect();
    println!("args url {} names0 {:?}", url, names[0]);
    let output = Command::new("docker").arg("pull").arg(url).output().await?;
    println!("docker pull res {:?}", output);
    let output = Command::new("docker")
        .arg("run")
        .arg(nurl)
        .arg(params)
        .output()
        .await?;
    println!("docker run res {:?}", output);
    Ok(())
}

pub async fn _send_request(sn: String) -> Result<()> {
    let url = format!("http://81.68.122.162:8000?sn={}", sn);
    let res = reqwest::get(url.clone()).await?.text().await?;
    println!("url {} res {:?}", url, res);
    Ok(())
}
