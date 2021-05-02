use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

//use cosmwasm_std::{CanonicalAddr, Storage};
//use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};


#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct Tally {
    // Number of votes in favor
    pub yes: u64,
    // Number of votes against
    pub no: u64,
    // List of addresses of voters
    pub voters: HashSet<Vec<u8>>,
}
