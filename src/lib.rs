use anyhow::{Context, Result};
use fuel_core_p2p::codecs::gossipsub::GossipsubMessageHandler;
use fuel_core_p2p::codecs::request_response::RequestResponseMessageHandler;
use fuel_core_p2p::config::{convert_to_libp2p_keypair, Config, Initialized};
use fuel_core_p2p::gossipsub::messages::GossipsubBroadcastRequest;
use fuel_core_p2p::p2p_service::FuelP2PService;
use fuel_core_p2p::Multiaddr;
use fuel_core_types::fuel_tx::Transaction;
use std::num::NonZero;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

mod genesis;

use genesis::genesis_config;

pub trait DisplayExt {
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

pub fn setup_config<I>(reserved_nodes_arg: I) -> Result<Config<Initialized>>
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
    cfg.enable_mdns = true;

    cfg.max_txs_per_request = 0;
    cfg.max_block_size = NonZero::new(1).unwrap();
    cfg.max_connections_per_peer = None;
    cfg.max_headers_per_request = 0;
    cfg.heartbeat_check_interval = Duration::from_millis(10);
    cfg.tcp_port = 9099;

    let genesis = genesis_config();

    let cfg = cfg.init(genesis).context("Failed to initialize config")?;

    Ok(cfg)
}

pub fn get_reserved_nodes<I>(mut _args: I) -> Result<Vec<Multiaddr>>
where
    I: Iterator<Item = String>,
{
    let multiaddrs = vec![Multiaddr::from_str(
        "/ip4/0.0.0.0/tcp/30333/p2p/16Uiu2HAmSrcZnrxMd67boCU281VCh5WGmXm24eawy64YoyuzAE29",
    )?];
    Ok(multiaddrs)
}

#[derive(Debug, Clone)]
pub struct P2pService {
    pub sender: tokio::sync::mpsc::Sender<Transaction>,
}

impl P2pService {
    pub async fn new() -> anyhow::Result<Self> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1_000_000); // Channel with a buffer of 100

        let cfg = setup_config(vec![].into_iter())?;
        let (broadcast, _) = broadcast::channel(5);

        // Spawn background task to process transactions
        tokio::task::spawn(async move {
            let max_block_size = cfg.max_block_size.clone();
            let mut p2p_service = FuelP2PService::new(
                broadcast,
                cfg,
                GossipsubMessageHandler::new(),
                RequestResponseMessageHandler::new(max_block_size),
            )
            .await
            .unwrap();
            p2p_service.start().await.unwrap();

            while let Some(tx) = rx.recv().await {
                if let Err(e) =
                    p2p_service.publish_message(GossipsubBroadcastRequest::NewTx(Arc::new(tx)))
                {
                    eprintln!("Failed to publish transaction: {:?}", e);
                }
            }
        });

        Ok(Self { sender: tx })
    }

    /// Public function to submit transactions via channel
    pub async fn submit_tx(&self, tx: Transaction) -> Result<()> {
        self.sender.send(tx).await.map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }
}
