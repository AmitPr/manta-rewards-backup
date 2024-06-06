use crate::claim::claim_rewards;
use crate::distributing::distribute_native;
use crate::eligibility::{
    execute_update_minimum_eligible_weight, query_minimum_eligible_weight, MINIMUM_ELIGIBLE_WEIGHT,
};
use crate::rewards::query_user_rewards;
use crate::state::SQUAD_CONTRACT;
use crate::user_weights::{get_initial_weights, save_initial_weights, update_user_weights};
use common::cw::{Context, QueryContext};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
};
use cw2::set_contract_version;
use funds_distributor_api::error::DistributorResult;
use funds_distributor_api::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:funds-distributor";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> DistributorResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let squad_contract = deps.api.addr_validate(&msg.squad_contract)?;
    SQUAD_CONTRACT.save(deps.storage, &squad_contract)?;

    let minimum_eligible_weight = msg.minimum_eligible_weight.unwrap_or_default();
    MINIMUM_ELIGIBLE_WEIGHT.save(deps.storage, &minimum_eligible_weight)?;

    let initial_weights = get_initial_weights(deps.as_ref(), squad_contract).unwrap();
    let mut ctx = Context { deps, env, info };

    save_initial_weights(&mut ctx, initial_weights, minimum_eligible_weight)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> DistributorResult<Response> {
    let ctx = &mut Context { deps, env, info };
    match msg {
        ExecuteMsg::MemberChangedHook(msg) => update_user_weights(ctx, msg),
        ExecuteMsg::UpdateMinimumEligibleWeight(msg) => {
            execute_update_minimum_eligible_weight(ctx, msg)
        }
        ExecuteMsg::DistributeNative {} => distribute_native(ctx),
        ExecuteMsg::ClaimRewards(msg) => claim_rewards(ctx, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> DistributorResult<Binary> {
    let qctx = QueryContext { deps, env };

    let response = match msg {
        QueryMsg::UserRewards(params) => to_json_binary(&query_user_rewards(qctx, params)?)?,
        QueryMsg::MinimumEligibleWeight {} => {
            to_json_binary(&query_minimum_eligible_weight(qctx)?)?
        }
    };
    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> DistributorResult<Response> {
    SQUAD_CONTRACT.save(deps.storage, &deps.api.addr_validate(&msg.new_hook_src)?)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("action", "migrate"))
}
