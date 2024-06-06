use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use cw4::MemberDiff;

#[cw_serde]
pub struct UpdateUserWeightsMsg {
    /// New weights that the users have, after the change
    pub diffs: Vec<MemberDiff>,
}

#[cw_serde]
pub struct UpdateMinimumEligibleWeightMsg {
    /// New minimum weight that the user must have to be eligible for rewards distributions
    pub minimum_eligible_weight: Uint128,
}

#[cw_serde]
pub struct UserWeight {
    pub user: String,
    pub weight: Uint128,
}

#[cw_serde]
pub struct ClaimRewardsMsg {
    pub user: String,
    /// Native denominations to be claimed
    pub native_denoms: Option<Vec<String>>,
}

#[cw_serde]
pub struct UserRewardsParams {
    pub user: String,
    /// Native denominations to be queried for rewards
    pub native_denoms: Option<Vec<String>>,
}

#[cw_serde]
pub struct UserRewardsResponse {
    pub native_rewards: Vec<NativeReward>,
}

#[cw_serde]
pub struct MinimumEligibleWeightResponse {
    pub minimum_eligible_weight: Uint128,
}

#[cw_serde]
pub struct NativeReward {
    pub denom: String,
    pub amount: Uint128,
}
