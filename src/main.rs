use anyhow::Result;
use clap::Parser;
use docker_runner::{Docker, DockerRunner};
use hex::FromHex;
use simplelog::*;
use tokio::{join, process::Command};
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
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        simplelog::Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .expect("Failed to init logger");
    let args = Args::parse();
    log::info!("{:?} {}!", args.url, args.chain);

    let out_string = {
        let output = Command::new("curl")
            .arg("localhost/api/admin/getDeviceId")
            .output()
            .await?;

        if output.stdout.is_empty() {
            let buffer = std::fs::read_to_string("./pk.json")?;
            let obj = json::parse(&buffer)?;
            obj["pubkey"].as_str().unwrap_or_default().to_string()
        } else {
            String::from_utf8_lossy(&output.stdout).to_string()
        }
    };

    // Report health status every 60s
    log::info!("shell output {:?}", out_string);
    let health_report = async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Ok(resp) =
                reqwest::get(format!("http://81.68.122.162:8000/?sn={}", out_string)).await
            {
                if let Ok(body) = resp.text().await {
                    log::info!("Id register res = {:?}", body);
                }
            }
        }
    };

    let docker = Docker::connect_with_socket_defaults().unwrap();
    let runner = DockerRunner::new(
        docker,
        // Container max execution time 1 hour
        60 * 60 * 1,
        "runner_container".into(),
        "yes".into(),
        10,
    );
    let runner_for_gc = runner.clone();
    let gc = async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            runner_for_gc
                .clear_images_by_whitelist(vec![
                    // helium miner
                    "sha256:9f78fc7319572294768f78381ff58eef7c0e4d49605a9f994b2fab056463dce0",
                ])
                .await
                .expect("Failed to clear images");
            runner_for_gc
                .clear_timeout_containers()
                .await
                .expect("Failed to clear old containers");
        }
    };
    let chain_watcher = async move {
        log::info!("inspect_chain_event begin");
        inspect_chain_event(args.chain, args.url.unwrap_or_default(), args.topic, runner)
            .await
            .expect("Failed to run chain watcher");
    };
    join!(gc, chain_watcher, health_report);
    Ok(())
}

pub async fn inspect_chain_event(
    chain: String,
    _url: String,
    topic: String,
    runner: DockerRunner,
) -> Result<()> {
    let topic_hash = <[u8; 32]>::from_hex(topic)?;
    let transport = web3::transports::Http::new(&chain.clone())?;
    let mut web3 = web3::Web3::new(transport);
    let mut base_number = web3.eth().block_number().await?;
    log::info!("init block number {:?}", base_number);
    let mut dst_number = base_number;

    loop {
        while base_number >= dst_number {
            tokio::time::sleep(std::time::Duration::new(8, 0)).await;
            let maybe_num = web3.eth().block_number().await;
            if maybe_num.is_err() {
                break;
            } else {
                dst_number = maybe_num.unwrap();
                log::info!("now dst block number {:?}", dst_number);
            }
        }

        let filter = FilterBuilder::default()
            .from_block(BlockNumber::Number(base_number))
            .to_block(BlockNumber::Number(dst_number))
            .topics(Some(vec![topic_hash.into()]), None, None, None)
            .build();

        let logs = web3.eth().logs(filter).await;
        if let Ok(logs) = logs {
            for log_content in logs {
                log::info!("got log: {:?}", log_content);
                let parse_res =
                    ethabi::decode(&[ParamType::String, ParamType::String], &log_content.data.0);
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
                let r = runner.clone();
                let image = strs[0].clone();
                let raw_cmd = strs[1].clone();
                tokio::spawn(async move {
                    log::info!("Got task: {} {}", image, raw_cmd);
                    let cmd = if raw_cmd == "" {
                        None
                    } else {
                        Some(raw_cmd.split(" ").collect())
                    };
                    if let Err(e) = r.run(image.as_str(), cmd).await {
                        log::warn!("Failed to run {} {}, {:?}", image, raw_cmd, e);
                    }
                });
            }

            base_number = dst_number + 1_u64;
        } else {
            let chain = chain.clone();
            loop {
                let reconn = web3::transports::Http::new(&chain.clone());
                log::info!("reconn res {:?}", reconn);
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
