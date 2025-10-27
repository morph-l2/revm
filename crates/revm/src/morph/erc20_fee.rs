use crate::primitives::{Bytes, TxEnv, TxKind};

use crate::morph::L1_GAS_PRICE_ORACLE_ADDRESS;
use crate::primitives::{address, Address, U256};
use crate::{Database, Evm};

// TokenAddressMappingSlot is the storage slot for mapping(uint16 => address)
const TOKEN_ADDRESS_MAPPING_SLOT: U256 = U256::from_limbs([1u64, 0, 0, 0]);
// TokenPriceMappingSlot is the storage slot for mapping(uint16 => uint256)
const TOKEN_PRICE_MAPPING_SLOT: U256 = U256::from_limbs([1u64, 0, 0, 0]);
// TokenBalanceSlotMappingSlot is the storage slot for mapping(uint16 => bytes32)
const TOKEN_BALANCE_SLOT_MAPPING_SLOT: U256 = U256::from_limbs([1u64, 0, 0, 0]);
// System address for receiving ERC20 fees
pub(super) const L2_FEE_VAULT: Address = address!("0e87cd091e091562F25CB1cf4641065dA2C049F5");

#[derive(Clone, Debug, Default)]
pub struct Erc20FeeInfo {
    /// The ERC20 token address
    pub token_address: Address,
    /// The price of the token
    pub price: U256,
    /// The caller address
    pub caller: Address,
    /// The token balance of caller
    pub balance: U256,
    /// The users' erc20 balance slot
    pub balance_slot: U256,
}

impl Erc20FeeInfo {
    // Get the token information for gas payment from the state db.
    pub(super) fn try_fetch<DB: Database>(
        db: &mut DB,
        token_id: u16,
        caller: Address,
    ) -> Result<Option<Erc20FeeInfo>, DB::Error> {
        // get token address of token_id
        let storage_value = load_mapping_value(
            db,
            L1_GAS_PRICE_ORACLE_ADDRESS,
            TOKEN_ADDRESS_MAPPING_SLOT,
            token_id.to_be_bytes().to_vec(),
        )?;
        let token_address: Address = Address::from_word(storage_value.into());
        if token_address.is_zero() {
            return Ok(None);
        }

        // get token price of token_id
        let token_price = load_mapping_value(
            db,
            L1_GAS_PRICE_ORACLE_ADDRESS,
            TOKEN_PRICE_MAPPING_SLOT,
            token_id.to_be_bytes().to_vec(),
        )?;
        if token_price.is_zero() {
            return Ok(None);
        }

        // get token balance of token_id
        let token_balance_slot = load_mapping_value(
            db,
            L1_GAS_PRICE_ORACLE_ADDRESS,
            TOKEN_BALANCE_SLOT_MAPPING_SLOT,
            token_id.to_be_bytes().to_vec(),
        )?;

        // get caller's token balance
        let caller_token_balance = get_erc20_balance(db, token_address, caller, token_balance_slot);

        let ecc20_fee = Erc20FeeInfo {
            token_address,
            price: token_price,
            caller,
            balance: caller_token_balance,
            balance_slot: token_balance_slot,
        };

        Ok(Some(ecc20_fee))
    }
}

fn load_mapping_value<DB: Database>(
    db: &mut DB,
    account: Address,
    slot_index: U256,
    mut key: Vec<u8>,
) -> Result<U256, <DB as Database>::Error> {
    let mut pre_image = slot_index.to_be_bytes_vec();
    pre_image.append(&mut key);
    let storage_key = crate::primitives::keccak256(pre_image);
    let storage_value = db.storage(account, U256::from_be_bytes(storage_key.0))?;
    Ok(storage_value)
}

pub(super) fn get_erc20_balance<DB: Database>(
    db: &mut DB,
    token: Address,
    account: Address,
    token_balance_slot: U256,
) -> U256 {
    // If balance slot is provided, try to read directly from storage
    if !token_balance_slot.is_zero() {
        if let Ok(balance) = load_mapping_value(db, token, token_balance_slot, account.to_vec()) {
            return balance;
        }
    }

    // Fallback: call balanceOf(address) method
    // Method signature: balanceOf(address) -> 0x70a08231
    let method_id = [0x70u8, 0xa0, 0x82, 0x31];
    
    // Encode calldata: method_id + padded address
    let mut calldata = Vec::with_capacity(36);
    calldata.extend_from_slice(&method_id);
    calldata.extend_from_slice(&[0u8; 12]); // Pad address to 32 bytes
    calldata.extend_from_slice(account.as_slice());

    // Create EVM instance and execute the call
    let mut evm = Evm::builder().with_db(db).build();
    let tx = TxEnv {
        caller: Address::default(),
        gas_limit: u64::MAX,
        transact_to: TxKind::Call(token),
        value: U256::ZERO,
        data: Bytes::from(calldata),
        nonce: None,
        chain_id: None,
        ..Default::default()
    };
    evm.context.evm.env.tx = tx;
    
    // Execute transaction and extract balance from output
    match evm.transact() {
        Ok(result) => {
            if result.result.is_success() {
                // Parse the returned balance (32 bytes)
                if let Some(output) = result.result.output() {
                    if output.len() >= 32 {
                        return U256::from_be_slice(&output[..32]);
                    }
                }
            }
            U256::ZERO
        }
        Err(_) => U256::ZERO,
    }
}

pub(super) fn transfer_erc20<DB: Database>(
    db: &mut DB,
    token: Address,
    from: Address,
    to: Address,
    amount: U256,
) {
    // Call transfer(address,uint256) method via EVM
    // Method signature: transfer(address,uint256) -> 0xa9059cbb
    let method_id = [0xa9u8, 0x05, 0x9c, 0xbb];
    
    // Encode calldata: method_id + padded to address + amount
    let mut calldata = Vec::with_capacity(68);
    calldata.extend_from_slice(&method_id);
    calldata.extend_from_slice(&[0u8; 12]); // Pad to address to 32 bytes
    calldata.extend_from_slice(to.as_slice());
    calldata.extend_from_slice(&amount.to_be_bytes::<32>());

    // Create EVM instance and execute the call
    let mut evm = Evm::builder().with_db(db).build();
    let tx = TxEnv {
        caller: from,
        gas_limit: u64::MAX,
        transact_to: TxKind::Call(token),
        value: U256::ZERO,
        data: Bytes::from(calldata),
        nonce: None,
        chain_id: None,
        ..Default::default()
    };
    evm.context.evm.env.tx = tx;
    let _ = evm.transact();
}
