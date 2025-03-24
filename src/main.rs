use std::num::NonZero;

use fuel_core_p2p::codecs::gossipsub::GossipsubMessageHandler;
use fuel_core_p2p::codecs::request_response::RequestResponseMessageHandler;
use fuel_core_p2p::p2p_service::{FuelP2PEvent, FuelP2PService};
use fuel_core_types::fuel_types::BlockHeight;
use fuel_gossiper::setup_config;
use tai64::Tai64N;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let args = std::env::args().skip(1);
    let cfg = setup_config(args)?;

    let (tx, _) = broadcast::channel(5);
    let mut p2p_service = FuelP2PService::new(
        tx,
        cfg,
        GossipsubMessageHandler::new(),
        RequestResponseMessageHandler::new(NonZero::new(0).unwrap()),
    )
    .await?;

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
