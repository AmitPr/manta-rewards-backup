use crate::state::NATIVE_GLOBAL_INDICES;
use crate::state::TOTAL_WEIGHT;
use common::cw::Context;
use cosmwasm_std::{Decimal, Response, Uint128};
use funds_distributor_api::error::DistributorError::ZeroTotalWeight;
use funds_distributor_api::error::DistributorResult;
use std::ops::Add;

/// Distributes new rewards for a native asset, using funds found in MessageInfo.
/// Will increase global index for each of the assets being distributed.
pub fn distribute_native(ctx: &mut Context) -> DistributorResult<Response> {
    let funds = ctx.info.funds.clone();

    let total_weight = TOTAL_WEIGHT.load(ctx.deps.storage)?;
    if total_weight == Uint128::zero() {
        return Err(ZeroTotalWeight);
    }

    for fund in funds {
        let global_index = NATIVE_GLOBAL_INDICES
            .may_load(ctx.deps.storage, fund.denom.clone())?
            .unwrap_or(Decimal::zero());

        // calculate how many units of the asset we're distributing per unit of total user weight
        // and add that to the global index for the asset
        let index_increment = Decimal::from_ratio(fund.amount, total_weight);

        NATIVE_GLOBAL_INDICES.save(
            ctx.deps.storage,
            fund.denom,
            &global_index.add(index_increment),
        )?;
    }

    Ok(Response::new()
        .add_attribute("action", "distribute_native")
        .add_attribute("total_weight", total_weight.to_string()))
}
