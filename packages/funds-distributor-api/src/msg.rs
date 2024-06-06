use crate::api::{
    ClaimRewardsMsg, MinimumEligibleWeightResponse, UpdateMinimumEligibleWeightMsg,
    UserRewardsParams, UserRewardsResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw4::MemberChangedHookMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub squad_contract: String,
    /// Optional minimum weight that the user must have to be eligible for rewards distributions
    pub minimum_eligible_weight: Option<Uint128>,
}

#[cw_serde]
pub enum ExecuteMsg {
    MemberChangedHook(MemberChangedHookMsg),
    UpdateMinimumEligibleWeight(UpdateMinimumEligibleWeightMsg),
    DistributeNative {},
    ClaimRewards(ClaimRewardsMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(UserRewardsResponse)]
    UserRewards(UserRewardsParams),
    #[returns(MinimumEligibleWeightResponse)]
    MinimumEligibleWeight {},
}

#[cw_serde]
pub struct MigrateMsg {
    pub new_hook_src: String,
}
