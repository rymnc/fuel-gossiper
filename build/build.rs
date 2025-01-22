use fuel_core_types::blockchain::consensus::Genesis;
use fuel_core_types::fuel_tx::Bytes32;
use std::fs;
use std::str::FromStr;

fn main() -> anyhow::Result<()> {
    // 1. the mainnet genesis config

    let genesis = Genesis {
        chain_config_hash: Bytes32::from_str(
            "0x5e8d733174398710cdafad299ac89b4ef4782cd303882a1cd30304ccf18c270a",
        )
        .map_err(|e| anyhow::anyhow!(e))?,
        coins_root: Bytes32::from_str(
            "0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        )
        .map_err(|e| anyhow::anyhow!(e))?,
        contracts_root: Bytes32::from_str(
            "0x70e4e3384ffe470a3802f0c1ff5fbb59fcea42329ef5bb9ef439d1db8853f438",
        )
        .map_err(|e| anyhow::anyhow!(e))?,
        messages_root: Bytes32::from_str(
            "0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        )
        .map_err(|e| anyhow::anyhow!(e))?,
        transactions_root: Bytes32::from_str(
            "0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        )
        .map_err(|e| anyhow::anyhow!(e))?,
    };

    // now because all are parsed, we can write them to a file for constant-time loading
    // Bytes32 = [u8; 32]

    let output = format!(
        "\n
    /// mainnet genesis block config
    pub const CHAIN_CONFIG_HASH: &[u8] = &{:?};
    pub const COINS_ROOT: &[u8] = &{:?};
    pub const CONTRACTS_ROOT: &[u8] = &{:?};
    pub const MESSAGES_ROOT: &[u8] = &{:?};
    pub const TRANSACTIONS_ROOT: &[u8] = &{:?};
    ",
        genesis.chain_config_hash.as_slice(),
        genesis.coins_root.as_slice(),
        genesis.contracts_root.as_slice(),
        genesis.messages_root.as_slice(),
        genesis.transactions_root.as_slice()
    );

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable not set");
    let dest_path = std::path::Path::new(&out_dir).join("genesis.rs");

    fs::write(&dest_path, output).map_err(|e| anyhow::anyhow!(e))?;

    // // 2. the multiaddrs
    // let reserved_nodes =  vec![
    //     "/dns/p2p-mainnet.fuel.network/tcp/30336/p2p/16Uiu2HAkxjhwNYtwawWUexYn84MsrA9ivFWkNHmiF4hSieoNP7Jd",
    //     "/dns/p2p-mainnet.fuel.network/tcp/30337/p2p/16Uiu2HAmQunK6Dd81BXh3rW2ZsszgviPgGMuHw39vv2XxbkuCfaw",
    //     "/dns/p2p-mainnet.fuel.network/tcp/30333/p2p/16Uiu2HAkuiLZNrfecgDYHJZV5LoEtCXqqRCqHY3yLBqs4LQk8jJg",
    //     "/dns/p2p-mainnet.fuel.network/tcp/30334/p2p/16Uiu2HAkzYNa6yMykppS1ij69mKoKjrZEr11oHGiM5Mpc8nKjVDM",
    //     "/dns/p2p-mainnet.fuel.network/tcp/30335/p2p/16Uiu2HAm5yqpTv1QVk3SepUYzeKXTWMuE2VqMWHq5qQLPR2Udg6s"
    // ].iter().map(|s| s.parse()).map(|r: Result<_, _>| r.unwrap()).collect::<Vec<Multiaddr>>();
    //
    //
    // let mut reserved_nodes_bytes = Vec::new();
    //
    // for node in reserved_nodes {
    //     let bytes = node.to_vec();
    //     reserved_nodes_bytes.push(bytes);
    // }
    //
    // let output = format!(
    //     "\n
    // /// reserved nodes
    // pub const RESERVED_NODES: &[[u8]] = &{:?};
    // pub fn reserved_nodes() -> Vec<fuel_core_p2p::Multiaddr> {{
    //     unsafe {{ std::mem::transmute_copy(RESERVED_NODES) }}
    // }}
    // ",
    //     reserved_nodes_bytes.iter().map(|inner| inner.as_slice()).collect::<Vec<&[u8]>>().as_slice()
    // );
    //
    //
    // let dest_path = std::path::Path::new(&out_dir).join("reserved_nodes.rs");
    //
    //
    // fs::write(&dest_path, output).map_err(|e| anyhow::anyhow!(e))?;
    //

    Ok(())
}
