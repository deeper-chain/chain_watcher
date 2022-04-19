use sp_core::Pair;
use sp_keyring::AccountKeyring;
use subxt::{ClientBuilder, DefaultConfig, PairSigner, PolkadotExtrinsicParams};

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod polkadot {}

mod test {
    use super::*;
    #[tokio::test]
    async fn test_sub() {
        let (pair, _) = sp_core::sr25519::Pair::from_phrase(
            "boring crush turtle chronic dignity taxi glide hill exist twenty sure movie",
            None,
            // Some("wdsr9198988"),
        )
        .unwrap();
        println!("{:?}", pair.public().to_string());

        let signer = PairSigner::new(pair);
        // let signer = PairSigner::new(AccountKeyring::Alice.pair());
        let dest = AccountKeyring::Bob.to_account_id().into();

        let api = ClientBuilder::new()
            .set_url("wss://mainnet-dev.deeper.network:443")
            .build()
            .await
            .unwrap()
            .to_runtime_api::<polkadot::RuntimeApi<DefaultConfig, PolkadotExtrinsicParams<_>>>();

        let balance_transfer = api
            .tx()
            .balances()
            .transfer(dest, 1_0000_0000_0000_0000_00)
            .sign_and_submit_then_watch_default(&signer)
            .await
            .unwrap()
            .wait_for_finalized_success()
            .await
            .unwrap();

        let transfer_event = balance_transfer
            // .fetch_events()
            // .await
            .find_first::<polkadot::balances::events::Transfer>()
            .unwrap();

        if let Some(event) = transfer_event {
            println!("Balance transfer success: {:?}", event);
        } else {
            println!("Failed to find Balances::Transfer Event");
        }
    }
}
