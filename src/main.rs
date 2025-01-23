use fuel_core_p2p::codecs::postcard::PostcardCodec;
use fuel_core_p2p::config::Config;
use fuel_core_p2p::p2p_service::{FuelP2PEvent, FuelP2PService};
use fuel_core_types::fuel_types::BlockHeight;
use tokio::sync::broadcast;

mod genesis;
mod reserved_nodes;

use genesis::genesis_config;
use reserved_nodes::reserved_nodes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut highest_block_height = 0;

    let genesis = genesis_config();

    let mut cfg = Config::default("Ignition");

    cfg.reserved_nodes_only_mode = true;
    cfg.reserved_nodes = reserved_nodes();

    let (tx, _) = broadcast::channel(5);
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
                                p2p_service.update_block_height(BlockHeight::from(highest_block_height));
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
