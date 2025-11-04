use crate::morph::L1_GAS_PRICE_ORACLE_ADDRESS;
use crate::primitives::{address, Address, U256};
use crate::primitives::{Bytes, TxEnv, TxKind};
use crate::{Database, Evm};

// TokenRegistry is the storage slot for mapping(uint16 => TokenInfo) - slot 0
const TOKEN_REGISTRY_SLOT: U256 = U256::from_limbs([0u64, 0, 0, 0]);
// PriceRatio is the storage slot for mapping(uint16 => uint256) - slot 2
const PRICE_RATIO_SLOT: U256 = U256::from_limbs([2u64, 0, 0, 0]);
// System address for receiving ERC20 fees
pub const L2_FEE_VAULT: Address = address!("0e87cd091e091562F25CB1cf4641065dA2C049F5");

#[derive(Clone, Debug, Default)]
pub struct Erc20FeeInfo {
    /// The ERC20 token address
    pub token_address: Address,
    /// Whether the token is active
    pub is_active: bool,
    /// Token decimals
    pub decimals: u8,
    /// The price ratio of the token
    pub price_ratio: U256,
    /// The scale of the token
    pub scale: U256,
    /// The caller address
    pub caller: Address,
    /// The token balance of caller
    pub balance: U256,
    /// The users' erc20 balance slot
    pub balance_slot: U256,
}

impl Erc20FeeInfo {
    // Get the token information for gas payment from the state db.
    pub fn try_fetch<DB: Database>(
        db: &mut DB,
        token_id: u16,
        caller: Address,
    ) -> Result<Option<Erc20FeeInfo>, DB::Error> {
        // Get the base slot for this token_id in tokenRegistry mapping
        let token_registry_base =
            get_mapping_slot(TOKEN_REGISTRY_SLOT, token_id.to_be_bytes().to_vec());

        // TokenInfo struct layout in storage (following Solidity storage packing rules):
        // slot + 0: tokenAddress (address, 20 bytes) + 12 bytes padding
        // slot + 1: balanceSlot (bytes32, 32 bytes)
        // slot + 2: isActive (bool, 1 byte) + decimals (uint8, 1 byte) + 30 bytes padding
        // slot + 3: scale (uint256, 32 bytes)

        // Read tokenAddress from slot + 0
        let slot_0 = db.storage(L1_GAS_PRICE_ORACLE_ADDRESS, token_registry_base)?;
        let token_address = Address::from_word(slot_0.into());

        // Read balanceSlot from slot + 1
        let token_balance_slot = db.storage(
            L1_GAS_PRICE_ORACLE_ADDRESS,
            token_registry_base + U256::from(1),
        )?;

        // Read isActive and decimals from slot + 2
        // In big-endian representation, rightmost byte is the lowest position
        // isActive is at the rightmost (byte 31), decimals is to its left (byte 30)
        let slot_2 = db.storage(
            L1_GAS_PRICE_ORACLE_ADDRESS,
            token_registry_base + U256::from(2),
        )?;
        let slot_2_bytes = slot_2.to_be_bytes::<32>();
        let is_active = slot_2_bytes[31] != 0;
        let decimals = slot_2_bytes[30];

        // Read scale from slot + 3
        let scale = db.storage(
            L1_GAS_PRICE_ORACLE_ADDRESS,
            token_registry_base + U256::from(3),
        )?;

        // Get price ratio from priceRatio mapping
        let price_ratio = load_mapping_value(
            db,
            L1_GAS_PRICE_ORACLE_ADDRESS,
            PRICE_RATIO_SLOT,
            token_id.to_be_bytes().to_vec(),
        )?;

        // Get caller's token balance
        let caller_token_balance = get_erc20_balance(db, token_address, caller, token_balance_slot);

        let erc20_fee = Erc20FeeInfo {
            token_address,
            is_active,
            decimals,
            price_ratio,
            scale,
            caller,
            balance: caller_token_balance,
            balance_slot: token_balance_slot,
        };

        Ok(Some(erc20_fee))
    }
}

/// Calculate the storage slot for a mapping value
fn get_mapping_slot(slot_index: U256, mut key: Vec<u8>) -> U256 {
    let mut pre_image = slot_index.to_be_bytes_vec();
    pre_image.append(&mut key);
    let storage_key = crate::primitives::keccak256(pre_image);
    U256::from_be_bytes(storage_key.0)
}

fn load_mapping_value<DB: Database>(
    db: &mut DB,
    account: Address,
    slot_index: U256,
    key: Vec<u8>,
) -> Result<U256, <DB as Database>::Error> {
    let storage_slot = get_mapping_slot(slot_index, key);
    let storage_value = db.storage(account, storage_slot)?;
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

    let db: &mut dyn Database<Error = DB::Error> = db;
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

pub fn eth_to_erc20(eth_amount: U256, rate: U256, token_scale: U256) -> U256 {
    if rate.is_zero() {
        return U256::ZERO;
    }
    // EthToERC20 erc20Amount = ethAmount / (tokenRate / tokenScale) = ethAmount * tokenScale / tokenRate
    // Calculate: (eth_amount * token_scale) / rate
    let (erc20_amount, remainder) = eth_amount.saturating_mul(token_scale).div_rem(rate);

    // If there's a remainder, round up by adding 1
    if !remainder.is_zero() {
        erc20_amount.saturating_add(U256::from(1))
    } else {
        erc20_amount
    }
}
