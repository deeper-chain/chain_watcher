use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::Wallet;
use ethers::signers::Signer;
use sp_core::Pair;
use subxt::{ClientBuilder, DefaultConfig, PairSigner, PolkadotExtrinsicParams};

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod polkadot {}

use polkadot::runtime_types::primitive_types::H160;
use polkadot::runtime_types::sp_core::ecdsa::Signature;

/// Substrate api convenience package
pub struct Substrate {
    pub api: polkadot::RuntimeApi<DefaultConfig, PolkadotExtrinsicParams<DefaultConfig>>,
}

impl Substrate {
    /// Use a ws or wss endpoint to build a substrate api
    /// Note that port must be specified
    pub async fn new(url: &str) -> Result<Self, anyhow::Error> {
        Ok(Self {
            api: ClientBuilder::new()
                .set_url(url)
                .build()
                .await?
                .to_runtime_api::<polkadot::RuntimeApi<DefaultConfig, PolkadotExtrinsicParams<_>>>(
                ),
        })
    }
    /// Pair a eth wallet to a substrate operator address, so that the evm can use operator's
    /// balance
    pub async fn pair_eth2sub(
        &self,
        eth_wallet: Wallet<SigningKey>,
        operator_substrate_phrase: &str,
    ) -> Result<(), anyhow::Error> {
        let (pair, _) = sp_core::sr25519::Pair::from_phrase(operator_substrate_phrase, None)
            .map_err(|e| anyhow::format_err!("{:?}", e))?;

        let operator_substrate_account_id = format!("{:x}", sp_core::H256::from(pair.public()));
        let signer = PairSigner::new(pair);
        let msg = format!("deeper evm:{}", operator_substrate_account_id,)
            .as_bytes()
            .to_vec();
        let signature = eth_wallet.sign_message(&msg).await.unwrap();
        let mut buffer = [0_u8; 20];
        eth_wallet
            .address()
            .as_bytes()
            .iter()
            .enumerate()
            .for_each(|(idx, el)| {
                buffer[idx] = el.clone();
            });
        let mut sig_buffer = [0_u8; 65];
        signature.to_vec().iter().enumerate().for_each(|(idx, el)| {
            sig_buffer[idx] = el.clone();
        });
        let pair_device_tx = self
            .api
            .tx()
            .evm()
            .device_pair_multi_accounts(H160(buffer), Signature(sig_buffer))
            .sign_and_submit_then_watch_default(&signer)
            .await?
            .wait_for_finalized_success()
            .await?;

        let pair_device_event =
            pair_device_tx.find_first::<polkadot::evm::events::DevicePairedAccounts>()?;

        if let Some(event) = pair_device_event {
            Ok(())
        } else {
            Err(anyhow::format_err!("Failed to find pair device transation"))
        }
    }
}

mod test {

    use super::*;
    use ethers::signers::LocalWallet;
    #[tokio::test]
    async fn test_sub() {
        let operator_substrate_phrase =
            "boring crush turtle chronic dignity taxi glide hill exist twenty sure movie";
        let endpoint = "wss://mainnet-dev.deeper.network:443";
        let wallet = LocalWallet::decrypt_keystore("./key/eth_key", "9527").unwrap();
        let sub = Substrate::new(endpoint).await.unwrap();
        if let Err(e) = sub.pair_eth2sub(wallet, operator_substrate_phrase).await {
            println!(
                "Already paired: {}",
                format!("{:?}", e).contains("EthAddressHasMapped")
            );
        } else {
            println!("Pair success");
        }
    }
}
