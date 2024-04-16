use bitcoin::secp256k1::{self, rand, Message, Secp256k1, Signing};
use bitcoin::sighash::{EcdsaSighashType, SighashCache};
use bitcoin::{
    absolute, transaction, Address, Amount, Network, OutPoint, PublicKey, Script, ScriptBuf,
    Sequence, Transaction, TxIn, TxOut, Txid, WPubkeyHash, Witness,
};

use bitcoind::bitcoincore_rpc::{Client, RpcApi};

mod setup;

const UTXO_AMOUNT: Amount = Amount::ONE_BTC;
const SPEND_AMOUNT: Amount = Amount::from_sat(50_000_000);
const CHANGE_AMOUNT: Amount = Amount::from_sat(49_999_000); // 1000 sat fee.

const NETWORK: Network = Network::Regtest;

#[test]
fn test_sign_segwit_v0() {
    let secp = Secp256k1::new();
    let cl = &setup::setup().client;

    // Keys controlled by us, the sender.
    let (sk, wpkh) = senders_keys(&secp);
    let pk = PublicKey::new(sk.public_key(&secp));

    // An unspent output that is locked to the key of the sender.
    let (out_point, utxo) = unspent_transaction_output(&cl, &pk);

    // Send coins back to the Bitcoin Core wallet.
    let address = cl.get_new_address(None, None).unwrap().assume_checked();

    // Create a transaction using the newly created utxo.
    // The input for the transaction we are constructing.
    let input = TxIn {
        previous_output: out_point,       // The output we are spending.
        script_sig: ScriptBuf::default(), // For a p2wpkh script_sig is empty.
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::default(), // Filled in after signing.
    };

    // The spend output is locked to a key controlled by the receiver.
    let spend = TxOut {
        value: SPEND_AMOUNT,
        script_pubkey: address.script_pubkey(),
    };

    // The change output is locked to a key controlled by us.
    let change = TxOut {
        value: CHANGE_AMOUNT,
        script_pubkey: ScriptBuf::new_p2wpkh(&wpkh), // Change comes back to us.
    };

    // The transaction we want to sign and broadcast.
    let mut unsigned_tx = Transaction {
        version: transaction::Version::TWO,  // Post BIP-68.
        lock_time: absolute::LockTime::ZERO, // Ignore the locktime.
        input: vec![input],                  // Input goes into index 0.
        output: vec![spend, change],         // Outputs, order does not matter.
    };
    let input_index = 0;

    // Get the sighash to sign.
    let sighash_type = EcdsaSighashType::All;
    let mut sighasher = SighashCache::new(&mut unsigned_tx);
    let sighash = sighasher
        .p2wpkh_signature_hash(input_index, &utxo.script_pubkey, UTXO_AMOUNT, sighash_type)
        .expect("failed to create sighash");

    // Sign the sighash using the secp256k1 library (exported by rust-bitcoin).
    let msg = Message::from(sighash);
    let signature = secp.sign_ecdsa(&msg, &sk);

    let pk = sk.public_key(&secp);
    assert!(pk.verify(&secp, &msg, &signature).is_ok());

    // Update the witness stack.
    let signature = bitcoin::ecdsa::Signature {
        sig: signature,
        hash_ty: sighash_type,
    };
    let pk = sk.public_key(&secp);
    *sighasher.witness_mut(input_index).unwrap() = Witness::p2wpkh(&signature, &pk);

    // Get the signed transaction.
    let tx = sighasher.transaction();

    // BOOM! Transaction signed and ready to broadcast.
    println!("{:#?}", tx);
    assert!(cl.send_raw_transaction(tx).is_ok())
}

/// A key controlled by the transaction sender.
fn senders_keys<C: Signing>(secp: &Secp256k1<C>) -> (secp256k1::SecretKey, WPubkeyHash) {
    let sk = secp256k1::SecretKey::new(&mut rand::thread_rng());
    let pk = bitcoin::PublicKey::new(sk.public_key(&secp));
    let wpkh = pk.wpubkey_hash().expect("key is compressed");

    (sk, wpkh)
}

/// Creates a p2wpkh output locked to the key associated with `pk`/`wpkh`.
fn unspent_transaction_output(cl: &Client, pk: &PublicKey) -> (OutPoint, TxOut) {
    let blocks = cl
        .generate_to_address(
            500,
            &cl.get_new_address(None, None).unwrap().assume_checked(),
        )
        .expect("failed to generate 500 blocks");
    assert_eq!(blocks.len(), 500);

    // Send some corn to an address associated with `pk` and `wpkh`.
    let me = Address::p2wpkh(&pk, NETWORK).expect("failed to create address");
    let spk = me.script_pubkey();

    let txid = cl
        .send_to_address(&me, UTXO_AMOUNT, None, None, None, None, None, None)
        .expect("failed to send one bitcoin");

    // Wait a few confirmations.
    let blocks = cl
        .generate_to_address(6, &cl.get_new_address(None, None).unwrap().assume_checked())
        .unwrap();
    assert_eq!(blocks.len(), 6);

    // Get the utxo that was just created.
    get_vout(&cl, txid, UTXO_AMOUNT, &spk)
}

// Find the Outpoint by spk
fn get_vout(cl: &Client, txid: Txid, value: Amount, spk: &Script) -> (OutPoint, TxOut) {
    let tx = cl
        .get_transaction(&txid, None)
        .unwrap()
        .transaction()
        .unwrap();

    for (i, txout) in tx.output.into_iter().enumerate() {
        if txout.value == value && spk == &txout.script_pubkey {
            return (OutPoint::new(txid, i as u32), txout);
        }
    }
    unreachable!("Only call get vout on functions which have the expected outpoint");
}
