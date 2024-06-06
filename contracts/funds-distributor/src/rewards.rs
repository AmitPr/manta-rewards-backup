use crate::native_distributions::NATIVE_DISTRIBUTIONS;
use crate::state::NATIVE_GLOBAL_INDICES;
use crate::user_weights::EFFECTIVE_USER_WEIGHTS;
use common::cw::QueryContext;
use cosmwasm_std::{Decimal, Order, Uint128};
use funds_distributor_api::api::{NativeReward, UserRewardsParams, UserRewardsResponse};
use funds_distributor_api::error::DistributorResult;
use std::collections::HashSet;
use std::ops::{Add, Mul, Sub};

/// Calculates user's currently available rewards for an asset, given its current global index
/// and user's weight.
pub fn calculate_user_reward(
    global_index: Decimal,
    distribution: Option<impl Into<(Decimal, Uint128)>>,
    user_weight: Uint128,
) -> Uint128 {
    let (user_index, pending_rewards) =
        distribution.map_or((Decimal::zero(), Uint128::zero()), |it| it.into());

    calculate_new_user_reward(global_index, user_index, user_weight).add(pending_rewards)
}

/// Calculates reward accrued for the given asset since the last update to the user's reward
/// index for the given asset.
pub fn calculate_new_user_reward(
    global_index: Decimal,
    user_index: Decimal,
    user_weight: Uint128,
) -> Uint128 {
    global_index.sub(user_index).mul(user_weight)
}

pub fn query_user_rewards(
    qctx: QueryContext,
    params: UserRewardsParams,
) -> DistributorResult<UserRewardsResponse> {
    let user = qctx.deps.api.addr_validate(&params.user)?;

    let user_weight = EFFECTIVE_USER_WEIGHTS
        .may_load(qctx.deps.storage, user.clone())?
        .unwrap_or_default();

    let mut native_rewards: Vec<NativeReward> = vec![];

    let denoms = params.native_denoms.map_or_else(
        || {
            NATIVE_GLOBAL_INDICES
                .keys(qctx.deps.storage, None, None, Order::Ascending)
                .collect::<Result<Vec<_>, _>>()
        },
        Ok,
    )?;
    let denoms = dedup_native_denoms(denoms);

    for denom in denoms {
        let global_index = NATIVE_GLOBAL_INDICES
            .may_load(qctx.deps.storage, denom.clone())?
            .unwrap_or_default();

        let distribution =
            NATIVE_DISTRIBUTIONS().may_load(qctx.deps.storage, (user.clone(), denom.clone()))?;

        let reward = calculate_user_reward(global_index, distribution, user_weight);

        native_rewards.push(NativeReward {
            denom,
            amount: reward,
        });
    }

    Ok(UserRewardsResponse { native_rewards })
}

/// Takes a vector of native denoms and returns a vector with all duplicates removed.
fn dedup_native_denoms(assets: Vec<String>) -> Vec<String> {
    let mut asset_set: HashSet<String> = HashSet::new();

    let mut deduped_assets: Vec<String> = vec![];

    for asset in assets {
        if !asset_set.contains(&asset) {
            asset_set.insert(asset.clone());
            deduped_assets.push(asset);
        }
    }

    deduped_assets
}
