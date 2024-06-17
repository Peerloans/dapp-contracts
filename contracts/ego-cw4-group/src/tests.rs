#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info,
    };
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn can_instantiate() {
        let mut deps = mock_dependencies();

        let res = instantiate_contract(deps.as_mut());
        assert_eq!(0, res.messages.len());

        let owner = &res
            .attributes
            .iter()
            .find(|a| a.key == "owner")
            .unwrap()
            .value;
        assert_eq!("creator", owner);
    }

    // #[test]
    // fn can_execute() {
    //     let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    //     instantiate_contract(deps.as_mut());

    //     let info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Dummy {};

    //     // TODO: fix this test when execute() is implemented
    //     let res = execute(deps.as_mut(), mock_env(), info, msg);
    //     match res {
    //         Err(ContractError::Std(StdError::GenericErr { msg })) => {
    //             assert_eq!("Not implemented", msg)
    //         }
    //         _ => panic!("Must return not implemented error"),
    //     }
    // }

    // #[test]
    // fn can_query() {
    //     let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    //     instantiate_contract(deps.as_mut());

    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::Hello {}).unwrap();
    //     let value: HelloResponse = from_binary(&res).unwrap();
    //     assert_eq!("Hello, Archway!", value.msg);
    // }

    // fn instantiate_contract(deps: DepsMut) -> Response {
    //     let msg = InstantiateMsg {};
    //     let info = mock_info("creator", &coins(1000, "token"));
    //     instantiate(deps, mock_env(), info, msg).unwrap()
    // }
}
