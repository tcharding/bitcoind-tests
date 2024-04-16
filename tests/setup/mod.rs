use bitcoind::bitcoincore_rpc::RpcApi;
use bitcoind::BitcoinD;

pub mod test_util;

// Launch an instance of bitcoind with
pub fn setup() -> BitcoinD {
    // Create env var BITCOIND_EXE_PATH to point to the ../bin/bitcoind binary
    let key = "BITCOIND_EXE";
    if std::env::var(key).is_err() {
        let mut root_path = std::env::current_dir().unwrap();
        while std::fs::metadata(root_path.join("LICENSE")).is_err() {
            if !root_path.pop() {
                panic!("Could not find LICENSE file; do not know where repo root is.");
            }
        }

        let bitcoind_path = root_path.join("bin").join("bitcoind");
        std::env::set_var(key, bitcoind_path);
    }

    let exe_path = bitcoind::exe_path().unwrap();
    let bitcoind = bitcoind::BitcoinD::new(exe_path).unwrap();
    let cl = &bitcoind.client;

    // Generate to an address for the wallet and wait for funds to mature.
    //
    // This also sanity checks that bitcoind is up and we can chat to it over RPC.
    let addr = cl.get_new_address(None, None).unwrap().assume_checked();
    let blks = cl.generate_to_address(101, &addr).unwrap();
    assert_eq!(blks.len(), 101);

    // Sanity check that we have coins ready to go.
    assert_eq!(
        cl.get_balance(Some(1) /*min conf*/, None).unwrap(),
        bitcoin::Amount::from_sat(100_000_000 * 50)
    );

    bitcoind
}
