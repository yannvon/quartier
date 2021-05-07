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
    // FIXME I would have liked to make it a HashMap but for some reason I couldn't make it work yet
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct Vote {
    // voted
    //pub has_voted: bool,
    // vote
    pub yes: bool,
    // time of vote
    pub timestamp: u64,
    // liquid
    //pub transfered: HumanAddress,
    // vote value (can be increased through transfered votes)
    //pub vote_value: u64,
}
