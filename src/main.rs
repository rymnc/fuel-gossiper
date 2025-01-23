use fuel_core_p2p::codecs::postcard::PostcardCodec;
use fuel_core_p2p::config::{convert_to_libp2p_keypair, Config};
use fuel_core_p2p::p2p_service::{FuelP2PEvent, FuelP2PService};
use fuel_core_types::fuel_types::BlockHeight;
use multiaddr::Multiaddr;
use std::str::FromStr;
use std::time::Duration;
use tai64::Tai64N;
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

    let keypair = std::env::var("KEYPAIR")?;
    let secret = fuel_core_types::fuel_crypto::SecretKey::from_str(&keypair)?;
    let keypair = convert_to_libp2p_keypair(&mut secret.to_vec())?;

    cfg.keypair = keypair;
    cfg.set_connection_keep_alive = Duration::from_secs(60);
    cfg.reserved_nodes = if args.len() >= 2 {
        let nodes = args.iter().skip(1).cloned().collect::<Vec<String>>();
        let multiaddrs = nodes
            .iter()
            .map(|node| Multiaddr::from_str(node).unwrap())
            .collect::<Vec<Multiaddr>>();
        tracing::info!("Connecting to overriden reserved nodes: {:?}", multiaddrs);
        multiaddrs
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
                if let Some(FuelP2PEvent::PeerInfoUpdated { block_height, .. }) = some_event {
                    let now = Tai64N::now();
                    // Update highest block height only if a new value is received
                    if *block_height > highest_block_height {
                        highest_block_height = *block_height;
                        tracing::info!("Highest block height: {}, rx at: {:?}", highest_block_height, now);

                        // Update P2P service with the new block height
                        p2p_service.update_block_height(BlockHeight::from(highest_block_height));
                    }
                }
            }
        }
    }

    Ok(())
}
