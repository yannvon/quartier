use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

//use cosmwasm_std::{CanonicalAddr, Storage};
//use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};


#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct Tally {
    pub yes: u64,
    pub no: u64,
}
