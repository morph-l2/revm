use crate::primitives::U256;
use revm_interpreter::gas;

use crate::{
    primitives::{db::Database, EVMError, Env, InvalidTransaction, Spec},
    Context,
};

/// Validate environment for the mainnet.
pub fn validate_env<SPEC: Spec, DB: Database>(env: &Env) -> Result<(), EVMError<DB::Error>> {
    // Important: validate block before tx.
    env.validate_block_env::<SPEC>()?;
    env.validate_tx::<SPEC>()?;
    Ok(())
}

/// Validates transaction against the state.
pub fn validate_tx_against_state<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
) -> Result<(), EVMError<DB::Error>> {
    // load acc
    let tx_caller = context.evm.env.tx.caller;
    let caller_account = context
        .evm
        .inner
        .journaled_state
        .load_code(tx_caller, &mut context.evm.inner.db)?;

    if context.evm.inner.env.tx.fee_token_id.unwrap_or_default() != 0
        && context.evm.inner.token_fee_info.is_none()
    {
        return Err(EVMError::Custom(
            "[MORPH] Failed to load token_fee_info.".to_string(),
        ));
    }

    let token_fee_info = match &context.evm.inner.token_fee_info {
        Some(fee_info) => (fee_info.balance, fee_info.price_ratio, fee_info.scale),
        None => (U256::ZERO, U256::ZERO, U256::ZERO),
    };

    context
        .evm
        .inner
        .env
        .validate_tx_against_state::<SPEC>(caller_account.data, token_fee_info)
        .map_err(EVMError::Transaction)?;

    Ok(())
}

/// Validate initial transaction gas.
pub fn validate_initial_tx_gas<SPEC: Spec, DB: Database>(
    env: &Env,
) -> Result<u64, EVMError<DB::Error>> {
    let input = &env.tx.data;
    let is_create = env.tx.transact_to.is_create();
    let access_list = &env.tx.access_list;
    let authorization_list_num = env
        .tx
        .authorization_list
        .as_ref()
        .map(|l| l.len() as u64)
        .unwrap_or_default();

    #[cfg(not(feature = "morph"))]
    let initial_gas_spend = gas::validate_initial_tx_gas(
        SPEC::SPEC_ID,
        input,
        is_create,
        access_list,
        authorization_list_num,
    );

    #[cfg(feature = "morph")]
    let initial_gas_spend = {
        let gas = gas::validate_initial_tx_gas(
            SPEC::SPEC_ID,
            input,
            is_create,
            access_list,
            authorization_list_num,
        );
        if gas > env.tx.gas_limit && env.tx.morph.is_l1_msg {
            env.tx.gas_limit
        } else {
            gas
        }
    };

    // Additional check to see if limit is big enough to cover initial gas.
    if initial_gas_spend > env.tx.gas_limit {
        return Err(InvalidTransaction::CallGasCostMoreThanGasLimit.into());
    }
    Ok(initial_gas_spend)
}
