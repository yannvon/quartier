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
    pub vote: Option<bool>,
    pub delegate: Option<HumanAddr>,
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
    Ballot {
        /// success or failure
        status: ResponseStatus,
        /// execution description
        message: String,
        // New vote
        #[serde(skip_serializing_if = "Option::is_none")]
        vote: Option<bool>,
        // Address of entity to which vote was delegated, called a delegate
        delegate: Option<HumanAddr>,
    },
    // generic status response
    Status {
        /// success or failure
        status: ResponseStatus,
        /// execution description
        message: String,
    },
}
