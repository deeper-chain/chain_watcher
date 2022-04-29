use anyhow::Result;
use core::panic;
use docker_runner::{Docker, DockerRunner};
use ethers::core::rand::thread_rng;
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::{Wallet, U256};
use ethers::signers::{LocalWallet, Signer};
use hex::FromHex;
use hex_literal::hex;
use secp256k1::SecretKey;
use serde::{Deserialize, Serialize};
use simplelog::*;
use std::default::Default;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use tokio::{join, process::Command};
use web3::contract::{Contract, Options};
use web3::{
    ethabi::{self, param_type::ParamType, Token},
    types::{BlockNumber, FilterBuilder},
};

// mod sub_client;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub operator: String,
    pub chain: String,
    pub topic: String,
    pub wallet_dir: String,
    pub wallet_filename: String,
    pub eth_key_password: String,
    pub operator_phrase: String,
    pub substrate_endpoint: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            operator: "5HmxV7yUHQnJYnVZVqDW2zd2qGznrvtqKLgyzFKnCS7jCAtT".into(),
            chain: "https://mainnet-dev.deeper.network/rpc".into(),
            topic: "f73572a14c820f867be454ea2eba1ec98b27da2ccc4e50373c3565b77bf6c569".into(),
            wallet_dir: "/var/deeper/web3d".into(),
            wallet_filename: "eth_key".into(),
            eth_key_password: "9527".into(),
            operator_phrase:
                "boring crush turtle chronic dignity taxi glide hill exist twenty sure movie".into(),
            substrate_endpoint: "wss://mainnet-dev.deeper.network:443".into(),
        }
    }
}

impl Config {
    pub fn from_yaml(config_path: &str) -> Result<Self, std::io::Error> {
        let file = File::open(config_path)?;
        let reader = BufReader::new(file);
        Ok(serde_yaml::from_reader(reader).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{:?}", e))
        })?)
    }
    pub fn to_yaml(&self, config_path: &str) -> Result<(), std::io::Error> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(config_path)?;
        let writer = BufWriter::new(file);
        serde_yaml::to_writer(writer, &self).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{:?}", e))
        })?;
        Ok(())
    }
    pub fn ensure_config(config_path: &str) -> Result<Self, std::io::Error> {
        match Self::from_yaml(config_path) {
            Ok(config) => {
                log::info!("Restore config from {}", config_path);
                Ok(config)
            }
            Err(e) => {
                log::info!("No valid config file found, using default config");
                log::debug!("Failed to load wallet: {:?}", e);
                let config = Self::default();
                config.to_yaml(config_path)?;
                Ok(config)
            }
        }
    }
}

/// Making sure wallet file exists and readable
fn ensure_wallet(
    wallet_dir: &str,
    wallet_filename: &str,
    password: &str,
) -> Result<LocalWallet, std::io::Error> {
    if !Path::new(wallet_dir).exists() {
        std::fs::create_dir_all(wallet_dir)?;
    }
    let key_path = Path::new(wallet_dir).join(wallet_filename);
    match LocalWallet::decrypt_keystore(&key_path, password) {
        Ok(wallet) => {
            log::info!("Restore evm wallet from key {:?}", key_path.to_str());
            Ok(wallet)
        }
        Err(e) => {
            if !key_path.exists() {
                log::info!("No existing key found, creating new evm wallet");
                let (wallet, filename) =
                    LocalWallet::new_keystore(&wallet_dir, &mut thread_rng(), password).unwrap();
                std::fs::rename(Path::new(wallet_dir).join(filename), &key_path)?;
                Ok(wallet)
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("{:?}", e),
                ))
            }
        }
    }
}

async fn get_sn() -> Result<String> {
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
    Ok(out_string)
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::ensure_config("/root/config.yaml")?;
    // let config = Config::ensure_config("./config.yaml")?;
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        simplelog::Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .expect("Failed to init logger");
    log::info!("{:?}", config);

    let wallet = ensure_wallet(
        &config.wallet_dir,
        &config.wallet_filename,
        &config.eth_key_password,
    )?;
    log::info!("Evm address: {:?}", wallet.address());
    log::info!("Evm private_key: {:x}", wallet.signer().to_bytes());
    let transport = web3::transports::Http::new(&config.chain.clone())?;
    let web3 = web3::Web3::new(transport);
    loop {
        let resp = reqwest::get(format!(
            "http://81.68.122.162:8000/get_starting_dpr?wallet_address={:x}",
            wallet.address()
        ))
        .await?
        .text()
        .await
        .unwrap_or("".into());
        log::info!("Get starting dpr res = {}", resp);
        if !resp.contains("Ok") && !resp.contains("Account has more than 1 balance") {
            panic!("Failed to get starting dpr");
        }
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        let balance = web3.eth().balance(wallet.address(), None).await?;
        if balance / 1_000_000_000_000_000_000_u64 > 1_u64.into() {
            break;
        }
    }

    let out_string = get_sn().await?;

    // Report health status every 60s
    log::info!("SN: {:?}", out_string);
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
        inspect_chain_event(config.chain, config.topic, runner, wallet)
            .await
            .expect("Failed to run chain watcher");
    };
    join!(gc, chain_watcher, health_report);
    Ok(())
}

pub async fn race_for_task(
    web3: web3::Web3<web3::transports::Http>,
    task_id: U256,
    self_eth_wallet: Wallet<SigningKey>,
) -> Result<(), anyhow::Error> {
    let contract = Contract::from_json(
        web3.eth(),
        hex!("48D1E091bc5eB638697A70d615a206A27a789225").into(),
        include_bytes!("../token.json"),
    )?;
    let result = contract
        .signed_call(
            "raceSubIndexForTask",
            (task_id,),
            Options {
                gas: Some(140850_u64.into()),
                ..Options::default()
            },
            &SecretKey::from_slice(&self_eth_wallet.signer().to_bytes()).unwrap(),
        )
        .await?;
    log::info!("Race tx: {:?}", result);
    for retry_times in 0..2 {
        tokio::time::sleep(std::time::Duration::from_secs(20)).await;
        if let Some(tx) = web3.eth().transaction_receipt(result).await? {
            if let Some(status) = tx.status {
                if status == 1_u64.into() {
                    break;
                }
                if status == 0_u64.into() {
                    return Err(anyhow::anyhow!("Already raced for this"));
                }
            }
        } else {
            log::warn!("Tx: {:?} not found", result);
            if retry_times == 1 {
                return Err(anyhow::anyhow!("Race transaction not found"));
            }
        }
    }
    Ok(())
}

pub async fn inspect_chain_event(
    chain: String,
    topic: String,
    runner: DockerRunner,
    self_eth_wallet: Wallet<SigningKey>,
) -> Result<()> {
    let topic_hash = <[u8; 32]>::from_hex(topic)?;
    let transport = web3::transports::Http::new(&chain.clone())?;
    let mut web3 = web3::Web3::new(transport);
    let mut base_number = web3.eth().block_number().await?;
    // let mut base_number = U64::from(3818535_u64);

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
                let parse_res = ethabi::decode(
                    &[
                        ParamType::Uint(0),
                        ParamType::String,
                        ParamType::String,
                        ParamType::Uint(0),
                    ],
                    &log_content.data.0,
                );
                let parse_res = parse_res.unwrap_or_default();
                if let [Token::Uint(task_id), Token::String(image_url), Token::String(args), Token::Uint(_max_task_num)] =
                    parse_res.as_slice()
                {
                    let r = runner.clone();
                    let tid = task_id.clone();
                    let image = image_url.clone();
                    let raw_cmd = args.clone();
                    let eth_wallet = self_eth_wallet.clone();
                    let web3_c = web3.clone();
                    tokio::spawn(async move {
                        log::info!("Got task: {} {} {}", tid, image, raw_cmd);
                        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                        for _ in 0..25 {
                            match race_for_task(web3_c.clone(), tid, eth_wallet.clone()).await {
                                Ok(_) => {
                                    log::info!("Race success {}", tid);
                                    let cmd = if raw_cmd == "" {
                                        None
                                    } else {
                                        Some(raw_cmd.split(" ").collect())
                                    };
                                    if let Err(e) = r.run(image.as_str(), cmd).await {
                                        log::warn!(
                                            "Failed to run {} {} {}, {:?}",
                                            tid,
                                            image,
                                            raw_cmd,
                                            e
                                        );
                                    }
                                    break;
                                }
                                Err(e) => {
                                    log::error!("Failed to race for task: {:?}", e);
                                    if format!("{:?}", e).contains("Already raced for this") {
                                        break;
                                    }
                                }
                            }
                        }
                    });
                } else {
                    log::info!("Skipping task: {:?}", parse_res);
                }
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
