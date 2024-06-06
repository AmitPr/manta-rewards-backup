use crate::eligibility::MINIMUM_ELIGIBLE_WEIGHT;
use crate::native_distributions;
use crate::native_distributions::{NativeDistribution, NATIVE_DISTRIBUTIONS};
use crate::state::{NATIVE_GLOBAL_INDICES, SQUAD_CONTRACT, TOTAL_WEIGHT};
use common::cw::Context;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{
    to_binary, Addr, Decimal, Deps, QueryRequest, Response, StdResult, Uint128, WasmQuery,
};
use cw_storage_plus::Map;

use cw4::Cw4QueryMsg::ListMembers;
use cw4::{Member, MemberChangedHookMsg, MemberListResponse};
use funds_distributor_api::error::DistributorError::Unauthorized;
use funds_distributor_api::error::{DistributorError, DistributorResult};
use native_distributions::update_user_native_distributions;
use DistributorError::DuplicateInitialWeight;

pub const USER_WEIGHTS: Map<Addr, Uint128> = Map::new("user_weights");

/// Effective user weights are their weights when taking into account minimum eligible weight
/// for rewards.
/// This weight will be the same as user's real weight if they're over the minimum eligible weight,
/// or 0 if they are under the minimum.
pub const EFFECTIVE_USER_WEIGHTS: Map<Addr, Uint128> = Map::new("effective_user_weights");

/// Saves any initial weights given to the users.
///
/// Should only be called when the contract is 'fresh'.
/// Do *NOT* call after there have already been reward distributions.
pub fn save_initial_weights(
    ctx: &mut Context,
    initial_weights: Vec<Member>,
    minimum_eligible_weight: Uint128,
) -> DistributorResult<()> {
    let mut total_weight = TOTAL_WEIGHT.may_load(ctx.deps.storage)?.unwrap_or_default();

    for user_weight in initial_weights {
        let user = ctx.deps.api.addr_validate(&user_weight.addr)?;

        if USER_WEIGHTS.has(ctx.deps.storage, user.clone())
            || EFFECTIVE_USER_WEIGHTS.has(ctx.deps.storage, user.clone())
        {
            return Err(DuplicateInitialWeight);
        }

        USER_WEIGHTS.save(
            ctx.deps.storage,
            user.clone(),
            &Uint128::from(user_weight.weight),
        )?;

        let effective_user_weight =
            calculate_effective_weight(Uint128::from(user_weight.weight), minimum_eligible_weight);
        EFFECTIVE_USER_WEIGHTS.save(ctx.deps.storage, user, &effective_user_weight)?;

        total_weight += effective_user_weight;
    }

    TOTAL_WEIGHT.save(ctx.deps.storage, &total_weight)?;

    Ok(())
}

pub fn get_initial_weights(deps: Deps, squad_contract: Addr) -> StdResult<Vec<Member>> {
    let query_msg = ListMembers {
        start_after: None,
        limit: Some(30),
    };

    let mut query_response: MemberListResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: squad_contract.to_string(),
            msg: to_binary(&query_msg)?,
        }))?;

    let mut member_list: Vec<Member> = vec![];

    while !query_response.members.is_empty() {
        member_list.append(&mut query_response.members);
        let query_msg = ListMembers {
            start_after: Some(member_list.last().unwrap().addr.clone()),
            limit: Some(30),
        };

        query_response = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: squad_contract.to_string(),
            msg: to_binary(&query_msg)?,
        }))?;
    }

    Ok(member_list)
}

/// Updates the users' weights to new ones.
/// Will calculate any accrued rewards since the last update to their rewards.
pub fn update_user_weights(
    ctx: &mut Context,
    msg: MemberChangedHookMsg,
) -> DistributorResult<Response> {
    let squad_contract = SQUAD_CONTRACT.load(ctx.deps.storage)?;

    if ctx.info.sender != squad_contract {
        return Err(Unauthorized);
    }

    let mut total_weight = TOTAL_WEIGHT.load(ctx.deps.storage)?;

    let minimum_eligible_weight = MINIMUM_ELIGIBLE_WEIGHT.load(ctx.deps.storage)?;

    for user_weight_change in msg.diffs {
        let user = ctx.deps.api.addr_validate(&user_weight_change.key)?;

        let old_user_effective_weight =
            EFFECTIVE_USER_WEIGHTS.may_load(ctx.deps.storage, user.clone())?;

        match old_user_effective_weight {
            None => {
                // we have not encountered this user, so we need to ensure their distribution
                // indices are set to current global indices
                initialize_user_indices(ctx, user.clone())?;
            }
            Some(old_user_effective_weight) => {
                // the user already had their weight previously, so we use that weight
                // to calculate how many rewards for each asset they've accrued since we last
                // calculated their pending rewards
                update_user_native_distributions(
                    ctx.deps.branch(),
                    user.clone(),
                    old_user_effective_weight,
                )?;
            }
        };

        match user_weight_change.new {
            None => {
                USER_WEIGHTS.remove(ctx.deps.storage, user.clone());

                EFFECTIVE_USER_WEIGHTS.remove(ctx.deps.storage, user);

                let old_user_effective_weight = old_user_effective_weight.unwrap_or_default();

                total_weight -= old_user_effective_weight;
            }

            Some(new_user_weight) => {
                USER_WEIGHTS.save(
                    ctx.deps.storage,
                    user.clone(),
                    &Uint128::from(new_user_weight),
                )?;

                let effective_user_weight = calculate_effective_weight(
                    Uint128::from(new_user_weight),
                    minimum_eligible_weight,
                );
                EFFECTIVE_USER_WEIGHTS.save(ctx.deps.storage, user, &effective_user_weight)?;

                let old_user_effective_weight = old_user_effective_weight.unwrap_or_default();

                total_weight = total_weight - old_user_effective_weight + effective_user_weight;
            }
        };
    }

    TOTAL_WEIGHT.save(ctx.deps.storage, &total_weight)?;

    Ok(Response::new().add_attribute("action", "update_user_weights"))
}

/// Calculate user's effective rewards weight, given their actual weight and minimum weight for
/// rewards eligibility
fn calculate_effective_weight(weight: Uint128, minimum_eligible_weight: Uint128) -> Uint128 {
    if weight >= minimum_eligible_weight {
        weight
    } else {
        Uint128::zero()
    }
}

/// Called for users that we did not encounter previously.
///
/// Will initialize all their rewards for assets with existing distributions to 0, and set
/// their rewards indices to current global index for each asset.
fn initialize_user_indices(ctx: &mut Context, user: Addr) -> DistributorResult<()> {
    let native_global_indices = NATIVE_GLOBAL_INDICES
        .range(ctx.deps.storage, None, None, Ascending)
        .collect::<StdResult<Vec<(String, Decimal)>>>()?;

    for (denom, global_index) in native_global_indices {
        NATIVE_DISTRIBUTIONS().update(
            ctx.deps.storage,
            (user.clone(), denom.clone()),
            |distribution| -> StdResult<NativeDistribution> {
                match distribution {
                    None => Ok(NativeDistribution {
                        user: user.clone(),
                        denom,
                        user_index: global_index,
                        pending_rewards: Uint128::zero(),
                    }),
                    Some(distribution) => Ok(distribution),
                }
            },
        )?;
    }

    Ok(())
}
