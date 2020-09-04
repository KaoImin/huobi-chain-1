use super::types::{GetBalanceResponse, UpdateIntervalPayload};
use super::*;

#[test]
fn test_riscv() {
    let res = exec_txs!(
        u64::max_value(),
        600_000,
        ("riscv", "update_interval", UpdateIntervalPayload {
            interval: 5000,
        })
    );

    assert_eq!(res.proposer_balance, 160);
}
