use std::fmt::Debug;

use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, HandleResult, InitResponse, InitResult, Querier,
    QueryResult, StdError, StdResult, Storage,
};
use serde::{Deserialize, Serialize};

use crate::msg::{InitMsg, HandleMsg, QueryMsg};
use crate::state::{Tally};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> InitResult {
    deps.storage.set(b"poll", &serialize(&msg.poll)?);

    let new_tally = Tally { yes: 0, no: 0 };
    deps.storage.set(b"tally", &serialize(&new_tally)?);
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: HandleMsg,
) -> HandleResult {
    let mut tally: Tally = deserialize(&deps.storage.get(b"tally").unwrap())?;

    if msg.yes {
        tally.yes += 1;
    } else {
        tally.no += 1;
    }

    deps.storage.set(b"tally", &serialize(&tally)?);
    Ok(HandleResponse::default())
}


pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::GetPoll {} => {
            let poll: String = deserialize(&deps.storage.get(b"poll").unwrap())?;
            Ok(to_binary(&poll)?)
        }
        QueryMsg::GetTally {} => {
            let tally: Tally = deserialize(&deps.storage.get(b"tally").unwrap())?;
            Ok(to_binary(&tally)?)
        }
    }
}

fn serialize<T: Serialize + Debug>(value: &T) -> StdResult<Vec<u8>> {
    bincode2::serialize(value)
        .map_err(|_err| StdError::generic_err(format!("Failed to serialize object: {:?}", value)))
}

fn deserialize<'a, T: Deserialize<'a> + Debug>(data: &'a [u8]) -> StdResult<T> {
    bincode2::deserialize(data)
        .map_err(|_err| StdError::generic_err(format!("Failed to serialize object: {:?}", data)))
}

#[cfg(test)]
mod tests {
    //use super::*;
    // use cosmwasm_std::testing::{mock_dependencies, mock_env};
    //use cosmwasm_std::{coins, from_binary, StdError};

    #[test]
    fn proper_initialization() {

    }

    #[test]
    fn vote() {
        assert_eq!(1+1, 2)
    }

    #[test]
    fn test_query() {
        assert_eq!(4, 1+3)

    }
}
