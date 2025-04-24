use revm::{
    db::{CacheDB, EmptyDB},
    primitives::{address, keccak256, AccountInfo, Bytes, TxEnv, U256},
    Evm,
};

fn main() {
    let mut cache_db = CacheDB::new(EmptyDB::default());

    let account = address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
    let account_to = address!("70997970C51812dc3A010C7d01b50e0d17dc79C8");

    let acc_info = AccountInfo {
        nonce: 0_u64,
        balance: U256::from(1_000_000_000_000_000_000u128),
        code_hash: keccak256(Bytes::new()),
        code: None,
        code_size: 0,
    };
    cache_db.insert_account_info(account, acc_info.clone());
    let mut evm = Evm::builder().with_db(cache_db).build();

    let tx1 = TxEnv {
        caller: account,
        gas_limit: u64::MAX,
        transact_to: account_to.into(),
        value: U256::from(1_000u64),
        data: Bytes::new(),
        nonce: None,
        chain_id: None,
        ..Default::default()
    };

    let tx2 = TxEnv {
        caller: account,
        gas_limit: u64::MAX,
        transact_to: account_to.into(),
        value: U256::from(1_000u64),
        data: Bytes::new(),
        nonce: None,
        chain_id: None,
        ..Default::default()
    };

    // process txs in block
    evm.process_block(vec![tx1, tx2])
}
