Integration tests for rust-bitcoin 
==================================

Integration tests for the crates in the https://github.com/rust-bitcoin (ie., the `rust-bitcoin` org).

Uses [`bitcoind`](https://crates.io/crates/bitcoind) to run a local Bitcoin Core instance and
[`bitcoincore-rpc`](https://crates.io/crates/bitcoincore-rpc) to hit it over RPC.

Original work taken from `rust-miniscript/bitcoind-tests` - thanks Sanket!
