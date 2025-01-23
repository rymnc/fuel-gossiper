use fuel_core_p2p::Multiaddr;
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
    use fuel_core_types::fuel_tx::Bytes32;
    use fuel_core_types::blockchain::consensus::Genesis;


    /// mainnet genesis block config
    const unsafe fn from_slice_unchecked<const N: usize>(buf: &[u8]) -> [u8; N] {{
        let ptr = buf.as_ptr() as *const [u8; N];
        // Static assertions are not applicable to runtime length check (e.g. slices).
        // This is safe if the size of `bytes` is consistent to `N`
        *ptr
    }}
    const CHAIN_CONFIG_HASH: Bytes32 = unsafe {{ Bytes32::new(from_slice_unchecked(&{:?})) }};
    const COINS_ROOT: Bytes32 = unsafe {{ Bytes32::new(from_slice_unchecked(&{:?})) }};
    const CONTRACTS_ROOT: Bytes32 = unsafe {{ Bytes32::new(from_slice_unchecked(&{:?})) }};
    const MESSAGES_ROOT: Bytes32 = unsafe {{ Bytes32::new(from_slice_unchecked(&{:?})) }};
    const TRANSACTIONS_ROOT: Bytes32 = unsafe {{ Bytes32::new(from_slice_unchecked(&{:?})) }};

    #[inline]
    pub const fn genesis_config() -> Genesis {{
        Genesis {{
            chain_config_hash: CHAIN_CONFIG_HASH,
            coins_root: COINS_ROOT,
            contracts_root: CONTRACTS_ROOT,
            messages_root: MESSAGES_ROOT,
            transactions_root: TRANSACTIONS_ROOT,
        }}
    }}
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
    let reserved_node_a = Multiaddr::from_str("/dns/p2p-mainnet.fuel.network/tcp/30336/p2p/16Uiu2HAkxjhwNYtwawWUexYn84MsrA9ivFWkNHmiF4hSieoNP7Jd")?;
    let reserved_node_b = Multiaddr::from_str("/dns/p2p-mainnet.fuel.network/tcp/30337/p2p/16Uiu2HAmQunK6Dd81BXh3rW2ZsszgviPgGMuHw39vv2XxbkuCfaw")?;
    let reserved_node_c = Multiaddr::from_str( "/dns/p2p-mainnet.fuel.network/tcp/30333/p2p/16Uiu2HAkuiLZNrfecgDYHJZV5LoEtCXqqRCqHY3yLBqs4LQk8jJg")?;
    let reserved_node_d = Multiaddr::from_str( "/dns/p2p-mainnet.fuel.network/tcp/30334/p2p/16Uiu2HAkzYNa6yMykppS1ij69mKoKjrZEr11oHGiM5Mpc8nKjVDM")?;
    let reserved_node_e = Multiaddr::from_str( "/dns/p2p-mainnet.fuel.network/tcp/30335/p2p/16Uiu2HAm5yqpTv1QVk3SepUYzeKXTWMuE2VqMWHq5qQLPR2Udg6s")?;

    let reserved_node_a_bytes = reserved_node_a.to_vec();
    let reserved_node_b_bytes = reserved_node_b.to_vec();
    let reserved_node_c_bytes = reserved_node_c.to_vec();
    let reserved_node_d_bytes = reserved_node_d.to_vec();
    let reserved_node_e_bytes = reserved_node_e.to_vec();

    let output = format!(
        "\n
    const RESERVED_NODE_A: &'static [u8] = &{:?};
    const RESERVED_NODE_B: &'static [u8] = &{:?};
    const RESERVED_NODE_C: &'static [u8] = &{:?};
    const RESERVED_NODE_D: &'static [u8] = &{:?};
    const RESERVED_NODE_E: &'static [u8] = &{:?};
    use fuel_core_p2p::Multiaddr;
    pub fn reserved_nodes() -> Vec<Multiaddr> {{
        vec![Multiaddr::from_static(RESERVED_NODE_A), Multiaddr::from_static(RESERVED_NODE_B), Multiaddr::from_static(RESERVED_NODE_C), Multiaddr::from_static(RESERVED_NODE_D), Multiaddr::from_static(RESERVED_NODE_E)]
    }}
    ",
        reserved_node_a_bytes,
        reserved_node_b_bytes,
        reserved_node_c_bytes,
        reserved_node_d_bytes,
        reserved_node_e_bytes
    );

    let dest_path = std::path::Path::new(&out_dir).join("reserved_nodes.rs");

    fs::write(&dest_path, output).map_err(|e| anyhow::anyhow!(e))?;

    Ok(())
}
