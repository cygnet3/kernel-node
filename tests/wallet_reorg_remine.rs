mod common;

use std::time::Duration;

use bitcoin::{Amount, BlockHash};
use common::{fund_silent_payment, start_bitcoind, wait_for_core_mempool, TestNode};

const SYNC_TIMEOUT: Duration = Duration::from_secs(60);
const REORG_TIMEOUT: Duration = Duration::from_secs(60);
const MEMPOOL_TIMEOUT: Duration = Duration::from_secs(30);
const PAYMENT: Amount = Amount::from_sat(100_000_000);

#[test]
fn reorg_drops_and_restores_a_re_mined_payment() {
    let core = start_bitcoind();
    let p2p = core.params.p2p_socket.unwrap();
    println!("step 1/7: bitcoind started on regtest");

    let node = TestNode::start_connected(p2p, Some(common::random_signing_keys()));
    let sp_address = node.receive_address();
    println!("step 2/7: node started, receive address {sp_address}");

    let funding_txid = fund_silent_payment(&core, &sp_address, PAYMENT);
    let funding_height = core.client.get_block_count().unwrap().0;
    let balance = node.wait_for_balance(PAYMENT, SYNC_TIMEOUT);
    assert_eq!(balance, PAYMENT);
    println!("step 3/7: payment mined at height {funding_height}, balance = {balance}");

    let funding_block = core
        .client
        .get_block_hash(funding_height)
        .unwrap()
        .0
        .parse::<BlockHash>()
        .unwrap();
    core.client.invalidate_block(funding_block).unwrap();
    wait_for_core_mempool(&core, &funding_txid.to_string(), MEMPOOL_TIMEOUT);
    println!("step 4/7: invalidated height {funding_height}, payment is back in the mempool");

    let miner = core.client.new_address().unwrap().to_string();
    core.client.generate_block(&miner, &[], true).unwrap();
    core.client.generate_block(&miner, &[], true).unwrap();
    let reorg_height = core.client.get_block_count().unwrap().0;
    let reorg_hash = core.client.best_block_hash().unwrap();
    assert!(reorg_height > funding_height);
    println!("step 5/7: chain extended past height {funding_height} without the payment");

    node.wait_for_tip(reorg_height, reorg_hash, REORG_TIMEOUT);
    node.wait_for_balance_of(Amount::ZERO, REORG_TIMEOUT);
    println!("step 6/7: node reorged, payment unconfirmed, balance = 0");

    core.client
        .generate_block(&miner, &[funding_txid.to_string()], true)
        .unwrap();
    let tip_height = core.client.get_block_count().unwrap().0;
    let tip_hash = core.client.best_block_hash().unwrap();

    node.wait_for_tip(tip_height, tip_hash, REORG_TIMEOUT);
    node.wait_for_balance_of(PAYMENT, REORG_TIMEOUT);
    println!("step 7/7: payment re-mined at height {tip_height}, balance restored to {PAYMENT}");

    node.stop();
}
