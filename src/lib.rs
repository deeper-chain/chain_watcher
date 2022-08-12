use anyhow::Result;
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::{Wallet, U256};
use ethers::signers::Signer;
use secp256k1::SecretKey;
use std::default::Default;
use std::fs::OpenOptions;
use std::io::Read;
use std::str::FromStr;
use web3::api::Eth;
use web3::contract::tokens::Tokenize;
use web3::contract::{Contract, Options};
use web3::transports::Http;
use web3::types::{Address, TransactionReceipt};

#[derive(Debug)]
pub struct Client {
    eth: Eth<Http>,
    contract: Contract<Http>,
    wallet: Wallet<SigningKey>,
}

impl Client {
    pub fn new(
        chain: &str,
        contract_addr: &str,
        abi_path: &str,
        wallet: Wallet<SigningKey>,
    ) -> Result<Self, anyhow::Error> {
        let transport = Http::new(chain)?;
        let web3 = web3::Web3::new(transport);
        let eth = web3.eth();
        let mut abi = vec![];
        OpenOptions::new()
            .read(true)
            .open(abi_path)?
            .read_to_end(&mut abi)?;
        let contract = Contract::from_json(eth.clone(), Address::from_str(contract_addr)?, &abi)?;
        Ok(Client {
            eth,
            contract,
            wallet,
        })
    }

    pub async fn read_u64(&self, func: &str) -> Result<(u64,), anyhow::Error> {
        let result: (u64,) = self
            .contract
            .query(
                func,
                (),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn task_info(
        &self,
        task_id: u64,
    ) -> Result<(U256, U256, U256, U256, U256, U256, U256, Address), anyhow::Error> {
        let result: (U256, U256, U256, U256, U256, U256, U256, Address) = self
            .contract
            .query(
                "taskInfo",
                (U256::from(task_id),),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn address_whitelist(&self, address: Address) -> Result<(bool,), anyhow::Error> {
        let result: (bool,) = self
            .contract
            .query(
                "addressWhitelist",
                (address,),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn block_unit_price(&self) -> Result<(u64,), anyhow::Error> {
        let result: (u64,) = self
            .contract
            .query(
                "blockUintPrice",
                (),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn complete_timeout(&self) -> Result<(u64,), anyhow::Error> {
        Ok(self.read_u64("completeTimeout").await?)
    }

    pub async fn credit_threshold(&self) -> Result<(u64,), anyhow::Error> {
        Ok(self.read_u64("creditThreshold").await?)
    }

    pub async fn day_total_reward(&self, day: u64) -> Result<(u64,), anyhow::Error> {
        let result: (u64,) = self
            .contract
            .query(
                "dayTotalReward",
                (U256::from(day),),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn estimate_run_num(&self) -> Result<(u64,), anyhow::Error> {
        Ok(self.read_u64("estimateRunNum").await?)
    }

    pub async fn get_current_time(&self) -> Result<(u64,), anyhow::Error> {
        Ok(self.read_u64("getCurrenTime").await?)
    }

    pub async fn get_current_day(&self) -> Result<(u64,), anyhow::Error> {
        Ok(self.read_u64("getCurrentDay").await?)
    }

    pub async fn get_sub_index_for_task(&self, task_id: u64) -> Result<(bool,), anyhow::Error> {
        let result: (bool,) = self
            .contract
            .query(
                "getSubIndexForTask",
                (U256::from(task_id),),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn get_task_remaining_time(&self, task_id: u64) -> Result<(u64,), anyhow::Error> {
        let result: (u64,) = self
            .contract
            .query(
                "getTaskRemainingTime",
                (U256::from(task_id),),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn get_total_reward_for_day(&self, the_day: u64) -> Result<(U256,), anyhow::Error> {
        let result: (U256,) = self
            .contract
            .query(
                "getTotalRewardForDay",
                (U256::from(the_day),),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn get_user_reward_for_current_day(
        &self,
        user: Address,
    ) -> Result<(U256,), anyhow::Error> {
        let result: (U256,) = self
            .contract
            .query(
                "getUserRewardForCurrentDay",
                (user,),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn get_user_reward_for_day(
        &self,
        user: Address,
        the_day: u64,
    ) -> Result<(U256,), anyhow::Error> {
        let result: (U256,) = self
            .contract
            .query(
                "getUserRewardForDay",
                (user, U256::from(the_day)),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn get_user_reward_pointer(&self, user: Address) -> Result<(u64,), anyhow::Error> {
        let result: (u64,) = self
            .contract
            .query(
                "getUserRewardPointer",
                (user,),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn image_whitelist_status(&self, image: &str) -> Result<(bool,), anyhow::Error> {
        let result: (bool,) = self
            .contract
            .query(
                "imageWhiteListStatus",
                (image.to_string(),),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn implementation_version(&self) -> Result<(String,), anyhow::Error> {
        let result: (String,) = self
            .contract
            .query(
                "implementationVersion",
                (),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn init_run_num(&self) -> Result<(u64,), anyhow::Error> {
        let result: (u64,) = self
            .contract
            .query(
                "initRunNum",
                (),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn is_withdraw_from_owner(&self, task_id: u64) -> Result<(bool,), anyhow::Error> {
        let result: (bool,) = self
            .contract
            .query(
                "isWithdrawFromOwner",
                (U256::from(task_id),),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn owner(&self) -> Result<(Address,), anyhow::Error> {
        let result: (Address,) = self
            .contract
            .query(
                "owner",
                (),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn proof_unit(&self) -> Result<(U256,), anyhow::Error> {
        let result: (U256,) = self
            .contract
            .query(
                "proofUnit",
                (),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn race_timeout(&self) -> Result<(U256,), anyhow::Error> {
        let result: (U256,) = self
            .contract
            .query(
                "raceTimeout",
                (),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn start_day(&self) -> Result<(U256,), anyhow::Error> {
        let result: (U256,) = self
            .contract
            .query(
                "startDay",
                (),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn task_sum(&self) -> Result<(u64,), anyhow::Error> {
        let result: (u64,) = self
            .contract
            .query(
                "taskSum",
                (),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn user_day_reward(
        &self,
        address: Address,
        day: u64,
    ) -> Result<(U256,), anyhow::Error> {
        let result: (U256,) = self
            .contract
            .query(
                "userDayReward",
                (address, day),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn user_reward_point(&self, user: Address) -> Result<(u64,), anyhow::Error> {
        let result: (u64,) = self
            .contract
            .query(
                "userRewardPoint",
                (user,),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn user_set_white_image(&self, user: Address) -> Result<(String,), anyhow::Error> {
        let result: (String,) = self
            .contract
            .query(
                "userSetWhiteImage",
                (user,),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn user_settled_day(&self, user: Address) -> Result<(u64,), anyhow::Error> {
        let result: (u64,) = self
            .contract
            .query(
                "userSettledDay",
                (user,),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn user_task(&self, user: Address, task_id: u64) -> Result<(bool,), anyhow::Error> {
        let result: (bool,) = self
            .contract
            .query(
                "userTask",
                (user, task_id),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    pub async fn user_task_completed(
        &self,
        user: Address,
        task_id: u64,
    ) -> Result<(bool,), anyhow::Error> {
        let result: (bool,) = self
            .contract
            .query(
                "userTask",
                (user, task_id),
                self.wallet.address(),
                Options {
                    gas: Some(140850_u64.into()),
                    ..Options::default()
                },
                None,
            )
            .await?;
        Ok(result)
    }

    async fn write_contract(
        &self,
        func: &str,
        params: impl Tokenize + Clone,
    ) -> Result<TransactionReceipt, anyhow::Error> {
        let nonce = self
            .eth
            .transaction_count(self.wallet.address(), None)
            .await?;
        let gas = self
            .contract
            .estimate_gas(
                func,
                params.clone(),
                self.wallet.address(),
                Options {
                    nonce: Some(nonce),
                    ..Options::default()
                },
            )
            .await?;
        let result = self
            .contract
            .signed_call_with_confirmations(
                func,
                params,
                Options {
                    gas: Some(gas),
                    nonce: Some(nonce),
                    ..Options::default()
                },
                1,
                &SecretKey::from_slice(&self.wallet.signer().to_bytes())?,
            )
            .await?;
        Ok(result)
    }

    pub async fn add_image_persistence_whitelist(
        &self,
        url: &str,
    ) -> Result<TransactionReceipt, anyhow::Error> {
        Ok(self
            .write_contract("addImagePersistenceWhitelist", (url.to_string(),))
            .await?)
    }

    pub async fn complete_sub_index_for_task(
        &self,
        task_id: u64,
    ) -> Result<TransactionReceipt, anyhow::Error> {
        Ok(self
            .write_contract("completeSubIndexForTask", (task_id,))
            .await?)
    }

    pub async fn delete_image(
        &self,
        image_hash: &str,
    ) -> Result<TransactionReceipt, anyhow::Error> {
        Ok(self
            .write_contract("deleteImage", (image_hash.to_string(),))
            .await?)
    }

    pub async fn increase_task_duration(
        &self,
        task_id: u64,
        maintain_extra_blocks: u64,
    ) -> Result<TransactionReceipt, anyhow::Error> {
        Ok(self
            .write_contract("increaseTaskDuration", (task_id, maintain_extra_blocks))
            .await?)
    }

    pub async fn n_node_unspecified_address_task(
        &self,
        url: &str,
        options: &str,
        max_run_num: u64,
        maintain_blocks: u64,
    ) -> Result<TransactionReceipt, anyhow::Error> {
        Ok(self
            .write_contract(
                "nNodeUnSpecifiedAddressTask",
                (
                    url.to_string(),
                    options.to_string(),
                    max_run_num,
                    maintain_blocks,
                ),
            )
            .await?)
    }

    pub async fn n_nodespecified_address_task(
        &self,
        url: &str,
        options: &str,
        max_run_num: u64,
        receivers: Vec<Address>,
        maintain_blocks: u64,
    ) -> Result<TransactionReceipt, anyhow::Error> {
        Ok(self
            .write_contract(
                "nNodespecifiedAddressTask",
                (
                    url.to_string(),
                    options.to_string(),
                    max_run_num,
                    receivers,
                    maintain_blocks,
                ),
            )
            .await?)
    }

    pub async fn race_sub_index_for_task(
        &self,
        task_id: u64,
    ) -> Result<TransactionReceipt, anyhow::Error> {
        Ok(self
            .write_contract("raceSubIndexForTask", (task_id,))
            .await?)
    }

    pub async fn reset_runners(
        &self,
        receivers: Vec<Address>,
    ) -> Result<TransactionReceipt, anyhow::Error> {
        Ok(self.write_contract("resetRunners", (receivers,)).await?)
    }

    pub async fn stop_task(&self, task_id: u64) -> Result<TransactionReceipt, anyhow::Error> {
        Ok(self.write_contract("stopTask", (task_id,)).await?)
    }

    pub async fn update_runner(&self, version: &str) -> Result<TransactionReceipt, anyhow::Error> {
        Ok(self
            .write_contract("updateRunner", (version.to_string(),))
            .await?)
    }

    pub async fn withdraw_ezc(&self, task_id: u64) -> Result<TransactionReceipt, anyhow::Error> {
        Ok(self.write_contract("withdrawEZC", (task_id,)).await?)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use ethers::signers::LocalWallet;
    use simplelog::*;
    #[tokio::test]
    async fn test_read_contract() {
        CombinedLogger::init(vec![TermLogger::new(
            LevelFilter::Info,
            simplelog::Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        )])
        .expect("Failed to init logger");
        let wallet = LocalWallet::decrypt_keystore("./eth.keystore", "VGPUmPKNtBzDvCJK").unwrap();
        log::info!("{:x}", wallet.address());
        let client = Client::new(
            "https://mainnet-dev.deeper.network/rpc",
            "9397AA12576cEc2A37C60f76d2FB31b31b5E5c7F",
            "./testnet.json",
            wallet,
        )
        .unwrap();
        log::info!("{:?}", client.task_info(1).await.unwrap());
        let address = Address::from_str("27FdDEF298618B512Fa6D281DB0e32E0F38D15D3").unwrap();
        log::info!("{:?}", client.address_whitelist(address).await.unwrap());
        log::info!("{:?}", client.block_unit_price().await.unwrap());
        log::info!("{:?}", client.complete_timeout().await.unwrap());
        log::info!("{:?}", client.credit_threshold().await.unwrap());
        log::info!("{:?}", client.day_total_reward(19215_u64).await.unwrap());
        log::info!("{:?}", client.get_current_time().await.unwrap());
        log::info!("{:?}", client.get_current_day().await.unwrap());
        log::info!("{:?}", client.estimate_run_num().await.unwrap());
        log::info!("{:?}", client.get_sub_index_for_task(1_u64).await.unwrap());
        log::info!("{:?}", client.get_task_remaining_time(1_u64).await.unwrap());
        log::info!(
            "{:?}",
            client.get_total_reward_for_day(1_u64).await.unwrap()
        );
        log::info!(
            "{:?}",
            client
                .get_user_reward_for_current_day(address)
                .await
                .unwrap()
        );
        log::info!(
            "{:?}",
            client
                .get_user_reward_for_day(address, 1_u64)
                .await
                .unwrap()
        );
        log::info!(
            "{:?}",
            client
                .image_whitelist_status("xbgxwh/oracle_price:1.0.3")
                .await
                .unwrap()
        );
        log::info!("{:?}", client.implementation_version().await.unwrap());
        log::info!("{:?}", client.init_run_num().await.unwrap());
        log::info!("{:?}", client.is_withdraw_from_owner(1_u64).await.unwrap());
        log::info!("{:?}", client.owner().await.unwrap());
        log::info!("{:?}", client.proof_unit().await.unwrap());
        log::info!("{:?}", client.race_timeout().await.unwrap());
        log::info!("{:?}", client.start_day().await.unwrap());
        log::info!("{:?}", client.task_sum().await.unwrap());
        log::info!(
            "{:?}",
            client.user_day_reward(address, 1_u64).await.unwrap()
        );
        log::info!("{:?}", client.user_reward_point(address).await.unwrap());
        log::info!("{:?}", client.user_set_white_image(address).await.unwrap());
        log::info!("{:?}", client.user_settled_day(address).await.unwrap());
        log::info!("{:?}", client.user_task(address, 1_u64).await.unwrap());
        log::info!(
            "{:?}",
            client.user_task_completed(address, 1_u64).await.unwrap()
        );
    }
}
