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
    tracing_subscriber::fmt().init();

    let mut highest_block_height = 0;

    let genesis = genesis_config();

    let mut cfg = Config::default("Ignition");

    cfg.reserved_nodes_only_mode = false;
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
            _ = tokio::signal::ctrl_c() => {
                println!("\nCtrl-C received, shutting down...");
                break;
            }
            some_event = p2p_service.next_event() => {
                if let Some(event) = some_event {
                    if let FuelP2PEvent::PeerInfoUpdated { block_height, .. } = event {
                        // Update highest block height only if a new value is received
                        if *block_height > highest_block_height {
                            highest_block_height = *block_height;
                            tracing::info!("Highest block height: {}", highest_block_height);

                            // Update P2P service with the new block height
                            p2p_service.update_block_height(BlockHeight::from(highest_block_height));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
