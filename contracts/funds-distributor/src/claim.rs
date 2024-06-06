use crate::native_distributions::{NativeDistribution, NATIVE_DISTRIBUTIONS};
use crate::rewards::calculate_user_reward;
use crate::state::NATIVE_GLOBAL_INDICES;
use crate::user_weights::EFFECTIVE_USER_WEIGHTS;
use common::cw::Context;
use cosmwasm_std::{coin, BankMsg, Order, Response, Uint128};
use cw_utils::NativeBalance;
use funds_distributor_api::api::ClaimRewardsMsg;
use funds_distributor_api::error::DistributorResult;

/// Attempt to claim rewards for the given parameters.
///
/// Calculates rewards currently available to the user, and marks them as claimed.
///
/// Returns a Response containing submessages that will send available rewards to the user.
pub fn claim_rewards(ctx: &mut Context, msg: ClaimRewardsMsg) -> DistributorResult<Response> {
    let user = ctx.deps.api.addr_validate(&msg.user)?;

    let user_weight = EFFECTIVE_USER_WEIGHTS
        .may_load(ctx.deps.storage, user.clone())?
        .unwrap_or_default();

    let denoms = msg.native_denoms.map_or_else(
        || {
            NATIVE_GLOBAL_INDICES
                .keys(ctx.deps.storage, None, None, Order::Ascending)
                .collect::<Result<Vec<_>, _>>()
        },
        Ok,
    )?;

    let mut coins = NativeBalance(vec![]);

    for denom in denoms {
        let distribution =
            NATIVE_DISTRIBUTIONS().may_load(ctx.deps.storage, (user.clone(), denom.clone()))?;
        let global_index = NATIVE_GLOBAL_INDICES
            .may_load(ctx.deps.storage, denom.clone())?
            .unwrap_or_default();

        // if no rewards for the given asset, just skip
        if global_index.is_zero() {
            continue;
        }

        let reward = calculate_user_reward(global_index, distribution, user_weight);
        coins += coin(reward.u128(), denom.clone());

        NATIVE_DISTRIBUTIONS().save(
            ctx.deps.storage,
            (user.clone(), denom.clone()),
            &NativeDistribution {
                user: user.clone(),
                denom,
                user_index: global_index,
                pending_rewards: Uint128::zero(),
            },
        )?;
    }

    coins.normalize();
    let coins = coins.into_vec();

    Ok(Response::new()
        .add_attribute("action", "claim_rewards")
        .add_attribute("user", user.to_string())
        .add_message(BankMsg::Send {
            to_address: user.to_string(),
            amount: coins,
        }))
}
