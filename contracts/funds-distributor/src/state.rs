use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};

pub const SQUAD_CONTRACT: Item<Addr> = Item::new("enterprise_contract");

/// Total weight of all users eligible for rewards.
pub const TOTAL_WEIGHT: Item<Uint128> = Item::new("total_weight");

/// Tracks global index for native denomination rewards.
/// Global index is simply a decimal number representing the amount of currency rewards paid
/// for a unit of user weight, since the beginning of time.
pub const NATIVE_GLOBAL_INDICES: Map<String, Decimal> = Map::new("native_global_indices");
