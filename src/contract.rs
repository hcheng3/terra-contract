#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{OwnerResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE, SCORE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:my-first-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::EnterScoreForToken{address, token, score} => try_enter_score(deps, info, address, token, score),
    }
}

pub fn try_enter_score(deps: DepsMut, info: MessageInfo, address:String, token:String, score: i32) -> Result<Response, ContractError>{
    let _address = deps.api.addr_validate(&address)?;
    let state = STATE.load(deps.storage)?;
    let replace_enty = |_a: Option<i32>| -> StdResult<_> { Ok(score) };
    if info.sender != state.owner {
        return Err(ContractError::Unauthorized {})
    }
    SCORE.update(deps.storage, (&_address, &token), replace_enty);

    Ok(Response::new().add_attribute("method", "try_enter_score"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetOwner {} => to_binary(&query_onwer(deps)?),
        QueryMsg::GetScore {address, token} => {
            let _address = deps.api.addr_validate(&address)?;
            let raw_score = SCORE.load(deps.storage, (&_address, &token))?;
            to_binary(&raw_score)
        },
    }
}

fn query_onwer(deps: Deps) -> StdResult<OwnerResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(OwnerResponse { owner: state.owner })
}


#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg { };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

    }

    #[test]
    fn enterScore() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

         // onwer can reset score
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::EnterScoreForToken {address:"terra1ptd53ng2fv38gd3x5665zzr478hpq3x8g8slc5".to_string(), token:"moon".to_string(), score: 5};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::EnterScoreForToken {address:"creator".to_string(), token:"Mirror".to_string(), score: 5};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        
        let res= query(deps.as_ref(), mock_env(), QueryMsg::GetScore {address: "creator".to_string() , token: "Mirror".to_string()}).unwrap();
        let scoreOfcreator:i32 = from_binary(&res).unwrap();
        assert_eq!(5, scoreOfcreator);

        //others cant reset score
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::EnterScoreForToken {address:"creator".to_string(), token:"USDT".to_string(), score: 5};
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }
    }

    #[test]
    fn onwerlookup() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg { };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // look up owner
        let info = mock_info("anyone", &coins(2, "token"));
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOwner {}).unwrap();
        let value: OwnerResponse = from_binary(&res).unwrap();
        assert_eq!("creator", value.owner);
    }

    #[test]
    fn scorelookup() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // look up for the score insert by owner
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::EnterScoreForToken {address:"creator".to_string(), token:"Mirror".to_string(), score: 5};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        
        let res= query(deps.as_ref(), mock_env(), QueryMsg::GetScore {address: "creator".to_string() , token: "Mirror".to_string()}).unwrap();
        let scoreOfcreator:i32 = from_binary(&res).unwrap();
        assert_eq!(5, scoreOfcreator);
    }
 
}
