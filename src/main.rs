use anyhow::{Context, Result};
use fuel_core_p2p::codecs::postcard::PostcardCodec;
use fuel_core_p2p::config::{convert_to_libp2p_keypair, Config, Initialized};
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

trait DisplayExt {
    fn display(&self) -> String;
}

impl DisplayExt for Vec<Multiaddr> {
    fn display(&self) -> String {
        self.iter()
            .map(|addr| addr.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    }
}

fn setup_config<I>(reserved_nodes_arg: I) -> Result<Config<Initialized>>
where
    I: Iterator<Item = String>,
{
    let mut cfg = Config::default("Ignition");

    cfg.reserved_nodes_only_mode = true;

    let keypair_str =
        std::env::var("KEYPAIR").context("Failed to read KEYPAIR from environment")?;
    let secret = fuel_core_types::fuel_crypto::SecretKey::from_str(&keypair_str)
        .context("Failed to parse secret key")?;
    let keypair = convert_to_libp2p_keypair(&mut secret.to_vec())
        .context("Failed to convert to libp2p keypair")?;

    cfg.keypair = keypair;
    cfg.set_connection_keep_alive = Duration::from_secs(60);
    cfg.reserved_nodes = get_reserved_nodes(reserved_nodes_arg)?;

    cfg.max_txs_per_request = 0;
    cfg.max_block_size = 0;
    cfg.max_connections_per_peer = 1;
    cfg.max_headers_per_request = 0;
    cfg.heartbeat_check_interval = Duration::from_millis(10);
    cfg.tcp_port = 9099;

    let genesis = genesis_config();

    let cfg = cfg.init(genesis).context("Failed to initialize config")?;

    Ok(cfg)
}

fn get_reserved_nodes<I>(mut args: I) -> Result<Vec<Multiaddr>>
where
    I: Iterator<Item = String>,
{
    if let Some(_) = args.next() {
        let mut multiaddrs = Vec::with_capacity(args.size_hint().0);
        for node in args {
            match Multiaddr::from_str(&node) {
                Ok(addr) => multiaddrs.push(addr),
                Err(e) => {
                    return Err(e).context("Failed to parse Multiaddr");
                }
            }
        }

        if multiaddrs.is_empty() {
            return Err(anyhow::anyhow!("No valid reserved nodes provided"));
        }

        tracing::info!(
            "Connecting to overridden reserved nodes: {}",
            multiaddrs.display()
        );
        Ok(multiaddrs)
    } else {
        Ok(reserved_nodes())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let args = std::env::args().skip(1);
    let cfg = setup_config(args)?;

    let (tx, _) = broadcast::channel(5);
    let mut p2p_service =
        FuelP2PService::new(tx, cfg, PostcardCodec::new(18 * 1024 * 1024)).await?;

    p2p_service.start().await?;

    let mut highest_block_height = 0;
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
