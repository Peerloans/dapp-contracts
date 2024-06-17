use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Dummy {},
}

#[cw_serde]
pub enum QueryMsg {
    Hello {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct HelloResponse {
    pub msg: String,
}
