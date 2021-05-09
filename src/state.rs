use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use cosmwasm_std::{
    HumanAddr,
};

//use cosmwasm_std::{CanonicalAddr, Storage};
//use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};


#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct Tally {
    // Number of votes in favor
    pub yes: u64,
    // Number of votes against
    pub no: u64,
    // List of addresses of voters
    pub voters: HashSet<Vec<u8>>,   // FIXME I would have liked to make it a HashMap but for some reason I couldn't make it work yet
    // Time of beginning of vote
    pub init_timestamp: u64,
    // Time of end of vote
    pub end_timestamp: u64,
    // Defines whether current tally state shall be private until completed
    pub early_results_allowed: bool,
    // Completion status, if true, that tally can be queried
    pub is_completed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct Ballot {
    // voted
    pub has_voted: bool,
    // time of vote
    pub timestamp: u64,
    // vote
    pub vote: Option<bool>,
    // allow liquid democracy
    pub delegate: Option<HumanAddr>,
    // vote value (can be increased through transfered votes)
    pub vote_value: u64,
}
