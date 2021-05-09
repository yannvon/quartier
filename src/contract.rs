use std::fmt::Debug;

use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, HandleResult, InitResponse, InitResult, Querier,
    QueryResult, StdError, StdResult, Storage,
};
use serde::{Deserialize, Serialize};

use crate::msg::{InitMsg, HandleMsg, QueryMsg, HandleAnswer, ResponseStatus::{Failure, Success},};
use crate::state::{Tally, Ballot};
use std::collections::HashSet;
use cosmwasm_std::{HumanAddr,};



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
    env: Env,
    msg: InitMsg,
) -> InitResult {
    deps.storage.set(b"poll", &serialize(&msg.poll)?);

    let new_tally = Tally { 
        yes: 0, 
        no: 0, 
        voters: HashSet::new(),
        init_timestamp: env.block.time,
        end_timestamp: msg.duration + env.block.time,
        is_completed: false,
        early_results_allowed: msg.early_results_allowed};
    
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
    
    // TODO add handle message to query current ballot

    let mut tally: Tally = deserialize(&deps.storage.get(b"tally").unwrap())?;
    let voter = &env.message.sender;
    let voter_raw = &deps.api.canonical_address(&voter)?;
    let mut message = String::new();
    //let mut vote: Option<bool> = msg.vote;
    //let mut delegate: Option<HumanAddr> = msg.delegate;
    let mut vote_value: u64 = 1;


    // First check that msg is valid, ie. it has either vote or delegate, but not both
    if (msg.vote.is_none() && msg.delegate.is_none()) || (!msg.vote.is_none() && !msg.delegate.is_none()) {
        
        // Malformed message
        // TODO better error message
        return Err(StdError::Unauthorized{backtrace: None})
    } 

    // First check whether Tally is still ongoing
    let current_timestamp: u64 = env.block.time;
    
    if tally.end_timestamp < current_timestamp {

        message.push_str("Tally is over. ");
        
        // Change is_completed boolean, such that now the result can be queried
        // TODO make is such this can be done without having to vote
        tally.is_completed = true;
        deps.storage.set(b"tally", &serialize(&tally)?);

        let mut vote: Option<bool> = None;
        let mut delegate: Option<HumanAddr> = None;

        // Check whether a ballot has been recorded and if so return
        // it with an error message.
        if tally.voters.contains(&voter_raw.as_slice().to_vec()) {
            let ballot: Ballot = deserialize(&deps.storage.get(voter_raw.as_slice()).unwrap())?;
            vote = ballot.vote;
            delegate = ballot.delegate;
            message.push_str("Previous ballot was however recorded.");
        } else {
            message.push_str("Ballot was not taken into account.");
        }

        return Ok(HandleResponse {
            messages: vec![],
            log: vec![],
            data: Some(to_binary(&HandleAnswer::Ballot {
                status: Failure,
                message,
                vote,
                delegate
            })?),
        });
    }

    // Otherwise, Tally is still ongoing

    // Check if a ballot already exists
    if tally.voters.contains(&voter_raw.as_slice().to_vec()) {

        // Check whether it is because of increased vote value, or because a vote was already cast.
        let mut ballot: Ballot = deserialize(&deps.storage.get(voter_raw.as_slice()).unwrap())?;

        if !ballot.has_voted {

            // OPTION 1: Voting takes place with increased vote value.
            vote_value = ballot.vote_value;

            // Helper function that does the dirty work
            // First, if we have a vote, we vote
            match msg.vote {
                Some(v) => {
                    if v {
                        tally.yes += vote_value;
                    } else {
                        tally.no += vote_value;
                    }
                }
                None => {
                    tally = delegate_vote(deps, &env, &msg, tally, &voter, vote_value, &msg.delegate)?
                }
            }
            
            message.push_str("Ballot was cast successfully!");

            // Save new and final ballot
            ballot.has_voted = true;
            ballot.timestamp = env.block.time;
            ballot.vote = msg.vote;
            ballot.delegate = msg.delegate;
            deps.storage.set(voter_raw.as_slice(), &serialize(&ballot)?);

            // Finally store updated tally
            deps.storage.set(b"tally", &serialize(&tally)?);

            return Ok(HandleResponse {
                messages: vec![],
                log: vec![],
                data: Some(to_binary(&HandleAnswer::Ballot {
                    status: Success,
                    message,
                    vote: msg.vote,
                    delegate: None, // FIXME return msg.delegate
                })?),
            });

        } else {

            // OPTION 2: Voter has already voted.
            
            // Important: previous feature of changing ballot is not possible in combination with liquid democracy: 
            // It could create a large chain of changes, with unpredictable cost. 
            // While in the current version delegating can create extra cost, it can be avoided by voting and not delegating.

            // In this version, as long as someone hasn't voted the vote value is increased,
            // but once a ballot is finalized, the tally is increased without increasing vote_value.

            message.push_str("Ballot was already cast!");
        
            return Ok(HandleResponse {
                messages: vec![],
                log: vec![],
                data: Some(to_binary(&HandleAnswer::Ballot {
                    status: Failure,
                    message,
                    vote: ballot.vote,
                    delegate: ballot.delegate
                })?),
            });
        }

    // Voter votes for the first time, and doesn't have increased vote value
    } else {
        
        // OPTION 3: Fresh ballot and single vote

        // Hard work is done by same helper function
        match msg.vote {
            Some(v) => {
                if v {
                    tally.yes += vote_value;
                } else {
                    tally.no += vote_value;
                }
            }
            None => {
                tally = delegate_vote(deps, &env, &msg, tally, &voter, vote_value, &msg.delegate)?
            }
        }

        message.push_str("Ballot was cast successfully!");
        
        // FIXME saving ballot needed ?
        // Add voter to list of voters to prevent double voting
        tally.voters.insert(voter_raw.as_slice().to_vec());

        // Create and save new ballot
        let new_ballot = Ballot {
            has_voted: true,
            timestamp: env.block.time,
            vote: msg.vote,
            delegate: msg.delegate, // FIXME add final delegate for future improvements
            vote_value: 1
        };
        deps.storage.set(voter_raw.as_slice(), &serialize(&new_ballot)?);

        // Finally store updated tally
        deps.storage.set(b"tally", &serialize(&tally)?);

        return Ok(HandleResponse {
            messages: vec![],
            log: vec![],
            data: Some(to_binary(&HandleAnswer::Ballot {
                status: Success,
                message,
                vote: msg.vote,
                delegate: None, // FIXME return real delegate. make sure not stored (optimized) delegate, but one chosen by voter
            })?),
        });
    }
}

// Sideffects: can create up to one new ballot, if final delegate has no ballot yet.
fn delegate_vote<S: Storage, A: Api, Q: Querier>(deps: &mut Extern<S, A, Q>, env: &Env, msg: &HandleMsg, 
    mut tally: Tally, voter: &HumanAddr, vote_value: u64, delegate: &Option<HumanAddr>) -> StdResult<Tally> {

    let voter_raw = &deps.api.canonical_address(&voter)?;
   

    match delegate{

        // No more delegation, end of recursion
        None => {

            // The current voter receives the voting power
            // Check if ballot already exists
            if tally.voters.contains(&voter_raw.as_slice().to_vec()) {
                let mut voter_ballot: Ballot = deserialize(&deps.storage.get(voter_raw.as_slice()).unwrap())?;

                // Check if already voted
                if voter_ballot.has_voted {
                    match voter_ballot.vote{
                        Some(v) => {
                            if v {
                                tally.yes += vote_value;
                            } else {
                                tally.no += vote_value;
                            }
                        }
                        None => {
                            panic!("unecpected error occurred.")
                        }
                    }
                } else {

                    // Simply increase vote value
                    voter_ballot.vote_value += vote_value;
                    deps.storage.set(voter_raw.as_slice(), &serialize(&voter_ballot)?);
                }
            } else {
                // Create ballot.
                let new_ballot = Ballot {
                    has_voted: false,
                    vote: None,
                    delegate: None,
                    timestamp: env.block.time,
                    vote_value: 1 + vote_value
                };
                deps.storage.set(voter_raw.as_slice(), &serialize(&new_ballot)?);

            } 
            return Ok(tally); 
        }
        Some(delegate) => {

            let delegate_raw = &deps.api.canonical_address(&delegate)?;


            // Keep recursion going
            if tally.voters.contains(&delegate_raw.as_slice().to_vec()) {
                let delegate_ballot: Ballot = deserialize(&deps.storage.get(&delegate_raw.as_slice()).unwrap())?;
                return delegate_vote(deps, env, msg, tally, &delegate, vote_value, &delegate_ballot.delegate)
            }
            else {
                return delegate_vote(deps, env, msg, tally, &delegate, vote_value, &None)
            }
            
        }  
    }
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
            
            // Check whether tally is over and thus can be disclosed
            if !tally.early_results_allowed && !tally.is_completed {
                return Err(StdError::Unauthorized{backtrace: None})
                // FIXME change to more informative answer
            }
            Ok(to_binary(&tally)?)
        }
 
        // Note: Querying vote makes no sense as we do not want to disclose it. 
        // One can always re-cast a vote in order to get the proof that it was counted. 
        
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
    //use std::thread;
    //use std::time;

    pub const STANDARD_DURATION: u64 = 10000000;
    
    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);  // canonical length, contract balance

        let msg = InitMsg { poll : String::from("Is the sky blue?"), duration: STANDARD_DURATION, early_results_allowed: true};
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

        let msg = InitMsg { poll : String::from("Is the sky blue?"), duration: STANDARD_DURATION, early_results_allowed: true};
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // anyone can vote
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg{ vote : Some(true), delegate: None};
        let _res = handle(&mut deps, env, msg).unwrap();

        // should increase yes tally by 1
        let res = query(&deps, QueryMsg::GetTally {}).unwrap();
        let value: Tally = from_binary(&res).unwrap();
        assert_eq!(1, value.yes);
        assert_eq!(0, value.no);

        // someone else can vote
        let env = mock_env("someone else", &coins(3, "token"));
        let msg = HandleMsg{ vote : Some(false), delegate: None};
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
        let msg = InitMsg { poll : String::from("Is the sky blue?"), duration: STANDARD_DURATION, early_results_allowed: true};
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // anyone can vote
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg{ vote : Some(true), delegate: None};
        let _res = handle(&mut deps, env, msg).unwrap();

        // lets figure out what the poll is
        let res = query(&deps, QueryMsg::GetPoll {}).unwrap();
        let value: String = from_binary(&res).unwrap();
        assert_eq!(value, String::from("Is the sky blue?"));
    }

    #[test]
    fn vote_only_counts_once() {
        let mut deps = mock_dependencies(20, &coins(2, "token")); // amount, denom

        let msg = InitMsg { poll : String::from("Is the sky blue?"), duration: STANDARD_DURATION, early_results_allowed: true };
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // anyone can vote
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg{ vote : Some(true), delegate: None};
        let _res = handle(&mut deps, env, msg).unwrap();

        // can vote twice, but should only count once though
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg{ vote : Some(true), delegate: None};
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
    fn cant_change_mind() {
        let mut deps = mock_dependencies(20, &coins(2, "token")); // amount, denom

        let msg = InitMsg { poll : String::from("Is the sky blue?"), duration: STANDARD_DURATION, early_results_allowed: true };
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // anyone can vote
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg{ vote : Some(true), delegate: None};
        let _res = handle(&mut deps, env, msg).unwrap();

        // cant change mind
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg{ vote : Some(false), delegate: None};
        let _res = handle(&mut deps, env, msg);

        // should increase true tally by 1 only
        let res = query(&deps, QueryMsg::GetTally {}).unwrap();
        let value: Tally = from_binary(&res).unwrap();
        assert_eq!(1, value.yes);
        assert_eq!(0, value.no);
    }

    #[test]
    fn no_more_voting_after_end() {
        // TODO
        // let two_seconds = time::Duration::from_millis(2000);
        // thread::sleep(two_seconds);
        // This doesn't change block, and thus not time of execution
    }

    #[test]
    fn secret_tally_is_revealed_after_end() {
        // TODO
    }

    #[test]
    fn simple_delegation() {
        // TODO
    }

    #[test]
    fn delegation_to_already_voted() {
        // TODO
    }

    #[test]
    fn delegation_to_new_addr() {
        // TODO
    }

    #[test]
    fn bad_vote_throws_error() {
        // TODO
    }


}
