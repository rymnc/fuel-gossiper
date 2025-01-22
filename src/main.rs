use fuel_core_p2p::codecs::postcard::PostcardCodec;
use fuel_core_p2p::config::Config;
use fuel_core_p2p::p2p_service::{FuelP2PEvent, FuelP2PService};
use fuel_core_types::blockchain::consensus::Genesis;
use fuel_core_types::fuel_tx::Bytes32;
use tokio::sync::broadcast;

mod genesis;

use genesis::*;

pub unsafe fn from_slice_unchecked<const N: usize>(buf: &[u8]) -> [u8; N] {
    let ptr = buf.as_ptr() as *const [u8; N];

    // Static assertions are not applicable to runtime length check (e.g. slices).
    // This is safe if the size of `bytes` is consistent to `N`
    *ptr
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut highest_block_height = 0;

    let genesis = Genesis {
        chain_config_hash: unsafe { Bytes32::new(from_slice_unchecked(&CHAIN_CONFIG_HASH)) },
        coins_root: unsafe { Bytes32::new(from_slice_unchecked(&COINS_ROOT)) },
        contracts_root: unsafe { Bytes32::new(from_slice_unchecked(&CONTRACTS_ROOT)) },
        messages_root: unsafe { Bytes32::new(from_slice_unchecked(&MESSAGES_ROOT)) },
        transactions_root: unsafe { Bytes32::new(from_slice_unchecked(&TRANSACTIONS_ROOT)) },
    };

    let mut cfg = Config::default("Ignition");

    cfg.reserved_nodes_only_mode = true;
    let reserved_nodes = vec![
        "/dns/p2p-mainnet.fuel.network/tcp/30336/p2p/16Uiu2HAkxjhwNYtwawWUexYn84MsrA9ivFWkNHmiF4hSieoNP7Jd",
        "/dns/p2p-mainnet.fuel.network/tcp/30337/p2p/16Uiu2HAmQunK6Dd81BXh3rW2ZsszgviPgGMuHw39vv2XxbkuCfaw",
        "/dns/p2p-mainnet.fuel.network/tcp/30333/p2p/16Uiu2HAkuiLZNrfecgDYHJZV5LoEtCXqqRCqHY3yLBqs4LQk8jJg",
        "/dns/p2p-mainnet.fuel.network/tcp/30334/p2p/16Uiu2HAkzYNa6yMykppS1ij69mKoKjrZEr11oHGiM5Mpc8nKjVDM",
        "/dns/p2p-mainnet.fuel.network/tcp/30335/p2p/16Uiu2HAm5yqpTv1QVk3SepUYzeKXTWMuE2VqMWHq5qQLPR2Udg6s"
    ].iter().map(|s| s.parse()).map(|r: Result<_, _>| r.unwrap()).collect();

    cfg.reserved_nodes = reserved_nodes;

    let (tx, _) = broadcast::channel(1);
    let mut p2p_service = FuelP2PService::new(
        tx,
        cfg.init(genesis).unwrap(),
        PostcardCodec::new(18 * 1024 * 1024),
    )
    .await?;

    p2p_service.start().await?;

    loop {
        tokio::select! {
            biased;

            _ = tokio::signal::ctrl_c() => {
                println!("\nCtrl-C received, shutting down...");
                break;
            }
            some_event = p2p_service.next_event() => {
                if let Some(event) = some_event {
                    match event {
                        FuelP2PEvent::PeerInfoUpdated{ block_height, .. } => {
                            if *block_height > highest_block_height {
                                highest_block_height = *block_height;
                                println!("{}", highest_block_height);
                            }
                        }
                        _ => {}
                    }

                }
            }
        }
    }

    Ok(())
}
