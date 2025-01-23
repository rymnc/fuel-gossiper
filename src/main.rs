use fuel_core_p2p::codecs::postcard::PostcardCodec;
use fuel_core_p2p::config::Config;
use fuel_core_p2p::p2p_service::{FuelP2PEvent, FuelP2PService};
use fuel_core_types::fuel_types::BlockHeight;
use multiaddr::Multiaddr;
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::broadcast;

mod genesis;
mod reserved_nodes;

use genesis::genesis_config;
use reserved_nodes::reserved_nodes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let args: Vec<String> = std::env::args().collect();

    let mut highest_block_height = 0;

    let genesis = genesis_config();

    let mut cfg = Config::default("Ignition");

    cfg.reserved_nodes_only_mode = true;
    cfg.set_connection_keep_alive = Duration::from_secs(60);
    cfg.reserved_nodes = if args.len() == 2 {
        let node_str = args.last().unwrap();
        let multiaddr = Multiaddr::from_str(node_str)?;
        tracing::info!("Connecting to overriden reserved node: {}", multiaddr);
        vec![multiaddr]
    } else {
        reserved_nodes()
    };

    cfg.max_txs_per_request = 0;
    cfg.max_block_size = 0;
    cfg.max_connections_per_peer = 1;
    cfg.max_headers_per_request = 0;
    cfg.heartbeat_check_interval = Duration::from_millis(50);
    cfg.tcp_port = 9099;

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
