use crate::primitives::{address, Address, Bytes};
use alloy_sol_types::{sol, SolInterface};
use IMorphToken::IMorphTokenCalls;
use L2Staking::L2StakingCalls;

use crate::{
    primitives::{db::Database, EVMError, U256},
    Context,
};
// SystemAddress is the address of the system
pub const SYSTEM_ADDRESS: Address = address!("5300000000000000000000000000000000000021");
pub const REWARD_EPOCH: U256 = U256::from_limbs([86400u64, 0, 0, 0]);

// MorphTokenAddress is the address of the morph token contract
pub const MORPH_TOKEN_ADDRESS: Address = address!("5300000000000000000000000000000000000013");
const INFLATION_MINTED_EPOCHS_SOLT: U256 = U256::from_limbs([8u64, 0, 0, 0]);

// L2StakingAddress is the address of the l2 staking contract
pub const L2_STAKING_ADDRESS: Address = address!("5300000000000000000000000000000000000015");
const REWARD_STARTED_SLOT: U256 = U256::from_limbs([1u64, 0, 0, 0]);
const REWARD_START_TIME_SLOT: U256 = U256::from_limbs([2u64, 0, 0, 0]);

sol! {
    #[derive(Debug)]
    contract L2Staking{
        function recordBlocks(address sequencerAddr) external;
    }

    #[derive(Debug)]
    interface IMorphToken{
        function mintInflations() external;
    }
}

pub fn load_reward_started<EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
) -> Result<U256, DB::Error> {
    let reward_started = context
        .evm
        .db
        .storage(L2_STAKING_ADDRESS, REWARD_STARTED_SLOT)?;

    Ok(reward_started)
}

pub fn load_inflation_minted_epochs<EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
) -> Result<U256, DB::Error> {
    let inflation_minted_epochs = context
        .evm
        .db
        .storage(MORPH_TOKEN_ADDRESS, INFLATION_MINTED_EPOCHS_SOLT)?;

    Ok(inflation_minted_epochs)
}

pub fn load_reward_start_time<EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
) -> Result<U256, DB::Error> {
    let reward_start_time = context
        .evm
        .db
        .storage(L2_STAKING_ADDRESS, REWARD_START_TIME_SLOT)?;

    Ok(reward_start_time)
}

// Generate transaction data for calling the recordBlocks method of the L2Staking contract
pub fn generate_record_blocks_input(sequencer_addr: Address) -> Bytes {
    // Create a call object
    let call = L2StakingCalls::recordBlocks(L2Staking::recordBlocksCall {
        sequencerAddr: sequencer_addr,
    });

    // Transaction data converted to bytes format
    call.abi_encode().into()
}

// Generate transaction data for calling the mintInflations method of the MorphToken contract
pub fn generate_mint_inflations_input() -> Bytes {
    // Create a call object
    let call = IMorphTokenCalls::mintInflations(IMorphToken::mintInflationsCall {});
    // Transaction data converted to bytes format
    call.abi_encode().into()
}
