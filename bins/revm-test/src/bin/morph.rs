use revm::{
    db::{CacheDB, EmptyDB},
    primitives::{address, bytes, keccak256, AccountInfo, Address, Bytecode, Bytes, TxEnv, U256},
    Database, Evm,
};

static ERC20_DEPLOYED_CODE : Bytes = bytes!("608060405234801561001057600080fd5b50600436106100a95760003560e01c80633950935111610071578063395093511461016857806370a082311461019857806395d89b41146101c8578063a457c2d7146101e6578063a9059cbb14610216578063dd62ed3e14610246576100a9565b806306fdde03146100ae578063095ea7b3146100cc57806318160ddd146100fc57806323b872dd1461011a578063313ce5671461014a575b600080fd5b6100b6610276565b6040516100c39190610b0c565b60405180910390f35b6100e660048036038101906100e19190610bc7565b610308565b6040516100f39190610c22565b60405180910390f35b61010461032b565b6040516101119190610c4c565b60405180910390f35b610134600480360381019061012f9190610c67565b610335565b6040516101419190610c22565b60405180910390f35b610152610364565b60405161015f9190610cd6565b60405180910390f35b610182600480360381019061017d9190610bc7565b61036d565b60405161018f9190610c22565b60405180910390f35b6101b260048036038101906101ad9190610cf1565b6103a4565b6040516101bf9190610c4c565b60405180910390f35b6101d06103ec565b6040516101dd9190610b0c565b60405180910390f35b61020060048036038101906101fb9190610bc7565b61047e565b60405161020d9190610c22565b60405180910390f35b610230600480360381019061022b9190610bc7565b6104f5565b60405161023d9190610c22565b60405180910390f35b610260600480360381019061025b9190610d1e565b610518565b60405161026d9190610c4c565b60405180910390f35b60606003805461028590610d8d565b80601f01602080910402602001604051908101604052809291908181526020018280546102b190610d8d565b80156102fe5780601f106102d3576101008083540402835291602001916102fe565b820191906000526020600020905b8154815290600101906020018083116102e157829003601f168201915b5050505050905090565b60008061031361059f565b90506103208185856105a7565b600191505092915050565b6000600254905090565b60008061034061059f565b905061034d858285610770565b6103588585856107fc565b60019150509392505050565b60006006905090565b60008061037861059f565b905061039981858561038a8589610518565b6103949190610ded565b6105a7565b600191505092915050565b60008060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020549050919050565b6060600480546103fb90610d8d565b80601f016020809104026020016040519081016040528092919081815260200182805461042790610d8d565b80156104745780601f1061044957610100808354040283529160200191610474565b820191906000526020600020905b81548152906001019060200180831161045757829003601f168201915b5050505050905090565b60008061048961059f565b905060006104978286610518565b9050838110156104dc576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016104d390610e93565b60405180910390fd5b6104e982868684036105a7565b60019250505092915050565b60008061050061059f565b905061050d8185856107fc565b600191505092915050565b6000600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054905092915050565b600033905090565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff1603610616576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161060d90610f25565b60405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1603610685576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161067c90610fb7565b60405180910390fd5b80600160008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020819055508173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925836040516107639190610c4c565b60405180910390a3505050565b600061077c8484610518565b90507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff81146107f657818110156107e8576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016107df90611023565b60405180910390fd5b6107f584848484036105a7565b5b50505050565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff160361086b576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610862906110b5565b60405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff16036108da576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016108d190611147565b60405180910390fd5b6108e5838383610a72565b60008060008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000205490508181101561096b576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610962906111d9565b60405180910390fd5b8181036000808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002081905550816000808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020600082825401925050819055508273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef84604051610a599190610c4c565b60405180910390a3610a6c848484610a77565b50505050565b505050565b505050565b600081519050919050565b600082825260208201905092915050565b60005b83811015610ab6578082015181840152602081019050610a9b565b60008484015250505050565b6000601f19601f8301169050919050565b6000610ade82610a7c565b610ae88185610a87565b9350610af8818560208601610a98565b610b0181610ac2565b840191505092915050565b60006020820190508181036000830152610b268184610ad3565b905092915050565b600080fd5b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b6000610b5e82610b33565b9050919050565b610b6e81610b53565b8114610b7957600080fd5b50565b600081359050610b8b81610b65565b92915050565b6000819050919050565b610ba481610b91565b8114610baf57600080fd5b50565b600081359050610bc181610b9b565b92915050565b60008060408385031215610bde57610bdd610b2e565b5b6000610bec85828601610b7c565b9250506020610bfd85828601610bb2565b9150509250929050565b60008115159050919050565b610c1c81610c07565b82525050565b6000602082019050610c376000830184610c13565b92915050565b610c4681610b91565b82525050565b6000602082019050610c616000830184610c3d565b92915050565b600080600060608486031215610c8057610c7f610b2e565b5b6000610c8e86828701610b7c565b9350506020610c9f86828701610b7c565b9250506040610cb086828701610bb2565b9150509250925092565b600060ff82169050919050565b610cd081610cba565b82525050565b6000602082019050610ceb6000830184610cc7565b92915050565b600060208284031215610d0757610d06610b2e565b5b6000610d1584828501610b7c565b91505092915050565b60008060408385031215610d3557610d34610b2e565b5b6000610d4385828601610b7c565b9250506020610d5485828601610b7c565b9150509250929050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052602260045260246000fd5b60006002820490506001821680610da557607f821691505b602082108103610db857610db7610d5e565b5b50919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b6000610df882610b91565b9150610e0383610b91565b9250828201905080821115610e1b57610e1a610dbe565b5b92915050565b7f45524332303a2064656372656173656420616c6c6f77616e63652062656c6f7760008201527f207a65726f000000000000000000000000000000000000000000000000000000602082015250565b6000610e7d602583610a87565b9150610e8882610e21565b604082019050919050565b60006020820190508181036000830152610eac81610e70565b9050919050565b7f45524332303a20617070726f76652066726f6d20746865207a65726f2061646460008201527f7265737300000000000000000000000000000000000000000000000000000000602082015250565b6000610f0f602483610a87565b9150610f1a82610eb3565b604082019050919050565b60006020820190508181036000830152610f3e81610f02565b9050919050565b7f45524332303a20617070726f766520746f20746865207a65726f20616464726560008201527f7373000000000000000000000000000000000000000000000000000000000000602082015250565b6000610fa1602283610a87565b9150610fac82610f45565b604082019050919050565b60006020820190508181036000830152610fd081610f94565b9050919050565b7f45524332303a20696e73756666696369656e7420616c6c6f77616e6365000000600082015250565b600061100d601d83610a87565b915061101882610fd7565b602082019050919050565b6000602082019050818103600083015261103c81611000565b9050919050565b7f45524332303a207472616e736665722066726f6d20746865207a65726f20616460008201527f6472657373000000000000000000000000000000000000000000000000000000602082015250565b600061109f602583610a87565b91506110aa82611043565b604082019050919050565b600060208201905081810360008301526110ce81611092565b9050919050565b7f45524332303a207472616e7366657220746f20746865207a65726f206164647260008201527f6573730000000000000000000000000000000000000000000000000000000000602082015250565b6000611131602383610a87565b915061113c826110d5565b604082019050919050565b6000602082019050818103600083015261116081611124565b9050919050565b7f45524332303a207472616e7366657220616d6f756e742065786365656473206260008201527f616c616e63650000000000000000000000000000000000000000000000000000602082015250565b60006111c3602683610a87565b91506111ce82611167565b604082019050919050565b600060208201905081810360008301526111f2816111b6565b905091905056fea2646970667358221220bd76a0877c61d26a928dd36a2ac3491d00e9086a429df7883853cc988a8c1cbf64736f6c63430008120033");

fn main() {
    eth_gas_normal();

    erc20_fee_normal();
}

fn erc20_fee_normal() {
    let account_from = address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
    let account_to = address!("70997970C51812dc3A010C7d01b50e0d17dc79C8");
    let mut cache_db = CacheDB::new(EmptyDB::default());
    let token_account = address!("fab77965cAfB593Bd86E2e8073407CAb7fD2f6c4");
    let token_account_info = AccountInfo {
        nonce: 0_u64,
        balance: U256::from(1_000_000_000_000_000_000u128),
        code_hash: keccak256(Bytes::new()),
        code: Some(Bytecode::new_legacy(ERC20_DEPLOYED_CODE.clone())),
        code_size: 0,
    };
    // cache_db.insert_contract(token_account);
    cache_db.insert_account_info(token_account, token_account_info.clone());

    // Calculate the storage location of account_from in the _balances mapping
    // Storage location of Solidity mapping = keccak256(abi.encode(key, slot))
    let balance_slot = U256::ZERO; // slot of _balances mapping in ERC20.
    let mut data = [0u8; 64];
    data[12..32].copy_from_slice(account_from.as_slice()); // The address occupies 20 bytes, left-padded to 32 bytes.
    data[32..64].copy_from_slice(&balance_slot.to_be_bytes::<32>()); // The slot occupies 32 bytes.

    let storage_key = keccak256(&data);
    let storage_key_u256 = U256::from_be_bytes(storage_key.0);

    // Set the balance to 200000000
    let balance_value = U256::from(200000000u64);
    let _ = cache_db.insert_account_storage(token_account, storage_key_u256, balance_value);

    // Set ERC20PriceOracle storage
    let oracle_address = address!("5300000000000000000000000000000000000021");
    let token_id: u16 = 1;

    // Calculate base slot for tokenRegistry[1]
    // tokenRegistry is at slot 0
    let token_registry_slot = U256::ZERO.to_be_bytes_vec();
    let mut token_id_bytes = [0u8; 32];
    token_id_bytes[30..32].copy_from_slice(&token_id.to_be_bytes());

    let mut token_registry_pre_image = token_id_bytes.to_vec();
    token_registry_pre_image.extend_from_slice(&token_registry_slot);
    let token_registry_base = keccak256(&token_registry_pre_image);
    let token_registry_base_u256 = U256::from_be_bytes(token_registry_base.0);

    // TokenInfo struct layout:
    // slot + 0: tokenAddress (address, 20 bytes) + 12 bytes padding
    // slot + 1: balanceSlot (bytes32, 32 bytes)
    // slot + 2: isActive (bool, 1 byte) + decimals (uint8, 1 byte) + 30 bytes padding
    // slot + 3: scale (uint256, 32 bytes)

    // Set tokenAddress at slot + 0
    let token_address_value = U256::from_be_bytes(token_account.into_word().into());
    let _ = cache_db.insert_account_storage(
        oracle_address,
        token_registry_base_u256,
        token_address_value,
    );

    // Set balanceSlot at slot + 1 (using slot 0 for ERC20 balance mapping)
    let balance_slot_value = U256::ZERO;
    let _ = cache_db.insert_account_storage(
        oracle_address,
        token_registry_base_u256 + U256::from(1),
        balance_slot_value,
    );

    // Set isActive and decimals at slot + 2
    // isActive = true (1), decimals = 18
    // In storage: rightmost byte (byte 31) is isActive, byte 30 is decimals
    let mut slot_2_bytes = [0u8; 32];
    slot_2_bytes[30] = 18; // decimals
    slot_2_bytes[31] = 1; // isActive = true
    let slot_2_value = U256::from_be_bytes(slot_2_bytes);
    let _ = cache_db.insert_account_storage(
        oracle_address,
        token_registry_base_u256 + U256::from(2),
        slot_2_value,
    );

    // Set scale at slot + 3
    // scale = 10
    let scale_value = U256::from(1u64);
    let _ = cache_db.insert_account_storage(
        oracle_address,
        token_registry_base_u256 + U256::from(3),
        scale_value,
    );

    // Set priceRatio for tokenID 1
    // priceRatio is at slot 2
    let price_ratio_slot = U256::from(2).to_be_bytes_vec();
    let mut price_ratio_pre_image = token_id_bytes.to_vec();
    price_ratio_pre_image.extend_from_slice(&price_ratio_slot);

    let price_ratio_storage_slot = keccak256(&price_ratio_pre_image);
    let price_ratio_storage_slot_u256 = U256::from_be_bytes(price_ratio_storage_slot.0);

    let price_ratio_value = U256::from(250000000u64);
    let _ = cache_db.insert_account_storage(
        oracle_address,
        price_ratio_storage_slot_u256,
        price_ratio_value,
    );

    let acc_info = AccountInfo {
        nonce: 0_u64,
        balance: U256::from(1_000_000_000_000_000_000u128),
        code_hash: keccak256(Bytes::new()),
        code: None,
        code_size: 0,
    };
    cache_db.insert_account_info(account_from, acc_info.clone());
    let mut evm = Evm::builder().with_db(&mut cache_db).build();

    // use erc20 gas token for txn.
    let mut tx = TxEnv {
        caller: account_from,
        gas_limit: 21000u64,
        transact_to: account_to.into(),
        value: U256::from(1_000u64),
        data: Bytes::new(),
        nonce: None,
        chain_id: None,
        fee_token_id: Some(1),
        fee_limit: None,
        gas_price: U256::from(10u64.pow(9)),
        ..Default::default()
    };

    // process txn
    tx.morph.is_l1_msg = false;
    tx.morph.rlp_bytes = Some(Bytes::default());
    evm.context.evm.env.tx = tx;
    let _ = evm.transact_commit();

    let account_from_balance = &evm
        .context
        .evm
        .inner
        .db
        .load_account(account_from)
        .unwrap()
        .info
        .balance;
    println!("account_from_balance: {:?}", account_from_balance);
    assert!(
        account_from_balance.to::<u64>() == 999999999999999000,
        "Only 1000wei must have been transferred."
    ); //Only the value 1_000 wei was transferred.

    let erc20_value = &evm
        .context
        .evm
        .db
        .storage(token_account, storage_key_u256)
        .unwrap_or_default();
    println!("account_from_erc20_value: {:?}", erc20_value);
    assert!(
        erc20_value.to::<u64>() == 199916000,
        "Gas fees should use: 84,000"
    ); //Gas fees used: 84,000

    let method_id = [0x70u8, 0xa0, 0x82, 0x31];
    let mut calldata = Vec::with_capacity(36);
    calldata.extend_from_slice(&method_id);
    calldata.extend_from_slice(&[0u8; 12]); // Pad address to 32 bytes
    calldata.extend_from_slice(account_from.as_slice());

    let mut token_balance_tx = TxEnv {
        caller: Address::default(),
        gas_limit: u64::MAX,
        transact_to: token_account.into(),
        value: U256::ZERO,
        data: Bytes::from(calldata),
        nonce: None,
        chain_id: None,
        ..Default::default()
    };
    token_balance_tx.morph.is_l1_msg = false;
    token_balance_tx.morph.rlp_bytes = Some(Bytes::default());
    evm.context.evm.env.tx = token_balance_tx;

    let erc20_balance_evm = match evm.transact() {
        Ok(result) => {
            if result.result.is_success() {
                // Parse the returned balance (32 bytes)
                if let Some(output) = result.result.output() {
                    if output.len() >= 32 {
                        U256::from_be_slice(&output[..32])
                    } else {
                        U256::ZERO
                    }
                } else {
                    U256::ZERO
                }
            } else {
                U256::ZERO
            }
        }
        Err(_) => {
            println!("get_erc20_balance error");
            U256::ZERO
        }
    };
    println!("account_from_erc20_value_evm: {:?}", erc20_balance_evm);

    assert!(erc20_value.eq(&erc20_balance_evm), "Gas fees used: 84,000") //Gas fees used: 84,000
}

fn eth_gas_normal() {
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

    // use erc20 gas token.
    let tx = TxEnv {
        caller: account,
        gas_limit: 21000u64,
        transact_to: account_to.into(),
        value: U256::from(1_000u64),
        data: Bytes::new(),
        nonce: None,
        chain_id: None,
        fee_token_id: Some(1u16),
        ..Default::default()
    };

    // process txn
    evm.context.evm.env.tx = tx;
    let _ = evm.transact_commit();
}
