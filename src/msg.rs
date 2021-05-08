use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{
    HumanAddr,
};


#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub poll: String,
    pub duration: u64,
    pub early_results_allowed: bool,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct HandleMsg {
    pub yes: bool,
    pub delegate: bool,
    pub voter: Option<HumanAddr>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetPoll {},
    GetTally {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TallyResponse {
    pub count: i32,
}

// success or failure response
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub enum ResponseStatus {
    Success,
    Failure,
}


// Responses from handle functions
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    // response from vote attempt
    Vote {
        /// success or failure
        status: ResponseStatus,
        /// execution description
        message: String,
        // Previous vote if there was any
        #[serde(skip_serializing_if = "Option::is_none")]
        previous_vote: Option<bool>,
        // New vote
        #[serde(skip_serializing_if = "Option::is_none")]
        vote_cast: Option<bool>,
    },
    // generic status response
    Status {
        /// success or failure
        status: ResponseStatus,
        /// execution description
        message: String,
    },
}
