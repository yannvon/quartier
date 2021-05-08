use std::fmt::Debug;

use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, HandleResult, InitResponse, InitResult, Querier,
    QueryResult, StdError, StdResult, Storage,
};
use serde::{Deserialize, Serialize};

use crate::msg::{InitMsg, HandleMsg, QueryMsg, HandleAnswer, ResponseStatus::{Failure, Success},};
use crate::state::{Tally, Vote};
use std::collections::HashSet;


// Disclaimer: The basic structure is taken from: https://github.com/enigmampc/SecretSimpleVote
// and is also inspired by https://github.com/baedrik/SCRT-sealed-bid-auction/blob/master/src/contract.rs


/// storage key for vote state
/// FIXME why do we distinguis poll and vote in existing code ?
// TODO
pub const VOTE_KEY: &[u8] = b"vote";
pub const POLL_KEY: &[u8] = b"poll";

/// pad handle responses and log attributes to blocks of 256 bytes to prevent leaking info based on
/// response size
// TODO
pub const BLOCK_SIZE: usize = 256;

////////////////////////////////////// Init ///////////////////////////////////////
/// Returns InitResult
///
/// Initializes the vote.
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `env` - Env of contract's environment
/// * `msg` - InitMsg passed in with the instantiation message
pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> InitResult {
    deps.storage.set(b"poll", &serialize(&msg.poll)?);

    let new_tally = Tally { yes: 0, no: 0, voters: HashSet::new()};
    deps.storage.set(b"tally", &serialize(&new_tally)?);
    Ok(InitResponse::default())
}


///////////////////////////////////// Handle //////////////////////////////////////
/// Returns HandleResult
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `env` - Env of contract's environment
/// * `msg` - HandleMsg passed in with the execute message
pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> HandleResult {
    let mut tally: Tally = deserialize(&deps.storage.get(b"tally").unwrap())?;
    let voter = env.message.sender;
    let voter_raw = &deps.api.canonical_address(&voter)?;
    let mut message = String::new();
    let mut previous_vote: Option<bool> = None;

    if tally.voters.contains(&voter_raw.as_slice().to_vec()) {
        // Voter has already cast a vote !
        let vote: Vote = deserialize(&deps.storage.get(voter_raw.as_slice()).unwrap())?;
        previous_vote = Some(vote.yes);

        // Check whether the tally needs to be adapted
        if msg.yes != vote.yes && msg.yes {
            // Previously voted no, now changed mind to yes
            tally.yes += 1;
            tally.no -= 1;
        } else if msg.yes != vote.yes && !msg.yes {
            // Previously voted yes, now changed mind to no
            tally.yes -= 1;
            tally.no += 1;
        }
        message.push_str(&format!("Vote changed to: {}", msg.yes));

    // Voter votes for the first time
    } else {
        message.push_str(&format!("Voted successully: {}", msg.yes));

        // Increase tally
        if msg.yes {
            tally.yes += 1;
        } else {
            tally.no += 1;
        }

        // Add voter to list of voters to prevent double voting
        tally.voters.insert(voter_raw.as_slice().to_vec());
    }

    // Save tally
    deps.storage.set(b"tally", &serialize(&tally)?);
    // Save latest vote
    let new_vote = Vote {
        yes: msg.yes,
        timestamp: env.block.time,
    };
    deps.storage.set(voter_raw.as_slice(), &serialize(&new_vote)?);


    Ok(HandleResponse {
      messages: vec![],
      log: vec![],
      data: Some(to_binary(&HandleAnswer::Vote {
          status: Success,
          message,
          previous_vote: previous_vote,
          vote_cast: msg.yes,
      })?),
  })
}


/////////////////////////////////////// Query /////////////////////////////////////
/// Returns QueryResult
///
/// # Arguments
///
/// * `deps` - reference to Extern containing all the contract's external dependencies
/// * `msg` - QueryMsg passed in with the query call
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
        QueryMsg::GetVote {} => {
            let tally: Tally = deserialize(&deps.storage.get(b"tally").unwrap())?;
            let voter = env.message.sender;
            let voter_raw = &deps.api.canonical_address(&voter)?;

            if !tally.voters.contains(&voter_raw.as_slice().to_vec()) {
                return  StdError // FIXME  
            }

            let vote: Vote = deserialize(&deps.storage.get(voter_raw.as_slice()).unwrap())?;
            Ok(to_binary(&vote)?)
        }

    }
}

pub fn serialize<T: Serialize + Debug>(value: &T) -> StdResult<Vec<u8>> {
    bincode2::serialize(value)
        .map_err(|_err| StdError::generic_err(format!("Failed to serialize object: {:?}", value)))
}

pub fn deserialize<'a, T: Deserialize<'a> + Debug>(data: &'a [u8]) -> StdResult<T> {
    bincode2::deserialize(data)
        .map_err(|_err| StdError::generic_err(format!("Failed to serialize object: {:?}", data)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);  // canonical length, contract balance

        let msg = InitMsg { poll : String::from("Is the sky blue?") };
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(&deps, QueryMsg::GetTally {}).unwrap();
        let value: Tally = from_binary(&res).unwrap();

        assert_eq!(0, value.yes);
        assert_eq!(0, value.no);
    }

    #[test]
    fn vote() {

        let mut deps = mock_dependencies(20, &coins(2, "token")); // amount, denom

        let msg = InitMsg { poll : String::from("Is the sky blue?") };
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // anyone can vote
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg{ yes : true, delegate: false, voter: None};
        let _res = handle(&mut deps, env, msg).unwrap();

        // should increase yes tally by 1
        let res = query(&deps, QueryMsg::GetTally {}).unwrap();
        let value: Tally = from_binary(&res).unwrap();
        assert_eq!(1, value.yes);
        assert_eq!(0, value.no);

        // someone else can vote
        let env = mock_env("someone else", &coins(3, "token"));
        let msg = HandleMsg{ yes : false, delegate: false, voter: None};
        let _res = handle(&mut deps, env, msg).unwrap();

        // should increase yes tally by 1
        let res = query(&deps, QueryMsg::GetTally {}).unwrap();
        let value: Tally = from_binary(&res).unwrap();
        assert_eq!(1, value.yes);
        assert_eq!(1, value.no);
    }

    #[test]
    fn poll() {
        let mut deps = mock_dependencies(20, &coins(2, "token")); // amount, denom

        // same setup
        let msg = InitMsg { poll : String::from("Is the sky blue?")};
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // anyone can vote
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg{ yes : true, delegate: false, voter: None};
        let _res = handle(&mut deps, env, msg).unwrap();

        // lets figure out what the poll is
        let res = query(&deps, QueryMsg::GetPoll {}).unwrap();
        let value: String = from_binary(&res).unwrap();
        assert_eq!(value, String::from("Is the sky blue?"));
    }

    #[test]
    fn vote_only_counts_once() {
        let mut deps = mock_dependencies(20, &coins(2, "token")); // amount, denom

        let msg = InitMsg { poll : String::from("Is the sky blue?") };
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // anyone can vote
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg{ yes : true, delegate: false, voter: None};
        let _res = handle(&mut deps, env, msg).unwrap();

        // can vote twice, but should only count once though
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg{ yes : true, delegate: false, voter: None};
        let _res = handle(&mut deps, env, msg);

        //match _res {
        //    Err(StdError::Unauthorized { .. }) => {}
        //        _ => panic!("Must disallow double vote"),
        //    }

        // should increase yes tally by 1 only
        let res = query(&deps, QueryMsg::GetTally {}).unwrap();
        let value: Tally = from_binary(&res).unwrap();
        assert_eq!(1, value.yes);
        assert_eq!(0, value.no);
    }

    #[test]
    fn can_change_mind() {
        let mut deps = mock_dependencies(20, &coins(2, "token")); // amount, denom

        let msg = InitMsg { poll : String::from("Is the sky blue?") };
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // anyone can vote
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg{ yes : true, delegate: false, voter: None};
        let _res = handle(&mut deps, env, msg).unwrap();

        // can change mind
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg{ yes : false, delegate: false, voter: None};
        let _res = handle(&mut deps, env, msg);

        // should increase no tally by 1 only
        let res = query(&deps, QueryMsg::GetTally {}).unwrap();
        let value: Tally = from_binary(&res).unwrap();
        assert_eq!(0, value.yes);
        assert_eq!(1, value.no);
    }
}
