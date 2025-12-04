use crate::primitives::{address, Address, EVMError, U256};
use crate::primitives::{keccak256, Bytes, TxEnv, TxKind};
use crate::{Database, Evm};

// https://github.com/morph-l2/morph/blob/main/contracts/contracts/l2/system/L2TokenRegistry.sol
// TokenRegistry is the storage slot for mapping(uint16 => TokenInfo) - slot 151
const TOKEN_REGISTRY_SLOT: U256 = U256::from_limbs([151u64, 0, 0, 0]);
// PriceRatio is the storage slot for mapping(uint16 => uint256) - slot 153
const PRICE_RATIO_SLOT: U256 = U256::from_limbs([153u64, 0, 0, 0]);
// System address for L2 token registry
pub const L2_TOKEN_REGISTRY_ADDRESS: Address = address!("5300000000000000000000000000000000000021");

#[derive(Clone, Debug, Default)]
pub struct TokenFeeInfo {
    /// The fee token address
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
    pub balance_slot: Option<U256>,
}

impl TokenFeeInfo {
    // Get the token information for gas payment from the state db.
    pub fn try_fetch<DB: Database>(
        db: &mut DB,
        token_id: u16,
        caller: Address,
    ) -> Result<Option<TokenFeeInfo>, DB::Error> {
        // Get the base slot for this token_id in tokenRegistry mapping
        let mut token_id_bytes = [0u8; 32];
        token_id_bytes[30..32].copy_from_slice(&token_id.to_be_bytes());
        let token_registry_base = get_mapping_slot(TOKEN_REGISTRY_SLOT, token_id_bytes.to_vec());
        // TokenInfo struct layout in storage (following Solidity storage packing rules):
        // slot + 0: tokenAddress (address, 20 bytes) + 12 bytes padding
        // slot + 1: balanceSlot (bytes32, 32 bytes)
        // slot + 2: isActive (bool, 1 byte) + decimals (uint8, 1 byte) + 30 bytes padding
        // slot + 3: scale (uint256, 32 bytes)

        // Read tokenAddress from slot + 0
        let slot_0 = db.storage(L2_TOKEN_REGISTRY_ADDRESS, token_registry_base)?;
        let token_address = Address::from_word(slot_0.into());
        if token_address == Address::default() {
            return Ok(None);
        }

        // Read balanceSlot from slot + 1
        let balance_slot_value = db.storage(
            L2_TOKEN_REGISTRY_ADDRESS,
            token_registry_base + U256::from(1),
        )?;
        let token_balance_slot = if !balance_slot_value.is_zero() {
            Some(balance_slot_value.saturating_sub(U256::from(1u64)))
        } else {
            None
        };

        // Read isActive and decimals from slot + 2
        // In big-endian representation, rightmost byte is the lowest position
        // isActive is at the rightmost (byte 31), decimals is to its left (byte 30)
        let slot_2 = db.storage(
            L2_TOKEN_REGISTRY_ADDRESS,
            token_registry_base + U256::from(2),
        )?;
        let slot_2_bytes = slot_2.to_be_bytes::<32>();
        let is_active = slot_2_bytes[31] != 0;
        let decimals = slot_2_bytes[30];

        // Read scale from slot + 3
        let scale = db.storage(
            L2_TOKEN_REGISTRY_ADDRESS,
            token_registry_base + U256::from(3),
        )?;

        // Get price ratio from priceRatio mapping
        let price_ratio = load_mapping_value(
            db,
            L2_TOKEN_REGISTRY_ADDRESS,
            PRICE_RATIO_SLOT,
            token_id_bytes.to_vec(),
        )?;

        // Get caller's token balance
        let caller_token_balance =
            get_erc20_balance(db, token_address, caller, token_balance_slot)?;
        let token_fee = TokenFeeInfo {
            token_address,
            is_active,
            decimals,
            price_ratio,
            scale,
            caller,
            balance: caller_token_balance,
            balance_slot: token_balance_slot,
        };
        Ok(Some(token_fee))
    }
}

/// Calculate the storage slot for a mapping value
pub fn get_mapping_slot(slot_index: U256, mut key: Vec<u8>) -> U256 {
    let mut pre_image = slot_index.to_be_bytes_vec();
    key.append(&mut pre_image);
    let storage_key = keccak256(key);
    U256::from_be_bytes(storage_key.0)
}

/// Calculate the account's storage slot for a mapping value
#[inline]
pub fn get_mapping_account_slot(slot_index: U256, account: Address) -> U256 {
    let mut key = [0u8; 32];
    key[12..32].copy_from_slice(account.as_slice());
    get_mapping_slot(slot_index, key.to_vec())
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
    token_balance_slot: Option<U256>,
) -> Result<U256, DB::Error> {
    // If balance slot is provided, try to read directly from storage
    if let Some(slot) = token_balance_slot {
        let mut data = [0u8; 32];
        data[12..32].copy_from_slice(account.as_slice());
        if let Ok(balance) = load_mapping_value(db, token, slot, data.to_vec()) {
            return Ok(balance);
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
    let mut tx = TxEnv {
        caller: Address::default(),
        gas_limit: u64::MAX,
        transact_to: TxKind::Call(token),
        value: U256::ZERO,
        data: Bytes::from(calldata),
        nonce: None,
        chain_id: None,
        ..Default::default()
    };
    tx.morph.is_l1_msg = true;
    tx.morph.rlp_bytes = Some(Bytes::default());
    evm.context.evm.env.tx = tx;

    // Execute transaction and extract balance from output
    match evm.transact() {
        Ok(result) => {
            if result.result.is_success() {
                // Parse the returned balance (32 bytes)
                if let Some(output) = result.result.output() {
                    if output.len() >= 32 {
                        return Ok(U256::from_be_slice(&output[..32]));
                    }
                }
            }
            Ok(U256::ZERO)
        }
        Err(EVMError::Database(db_err)) => {
            println!("get_erc20_balance db error");
            Err(db_err)
        }
        Err(e) => {
            match &e {
                EVMError::Transaction(t) => {
                    println!("get_erc20_balance Transaction error: {:?}", t)
                }
                EVMError::Header(h) => println!("get_erc20_balance Header error: {:?}", h),
                EVMError::Database(_) => println!("get_erc20_balance Database error"),
                EVMError::Custom(c) => println!("get_erc20_balance Custom error: {}", c),
                EVMError::Precompile(p) => println!("get_erc20_balance Precompile error: {}", p),
            }
            Ok(U256::ZERO)
        }
    }
}
