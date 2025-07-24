use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use bc::msg::{
    AccessLevel, ContractInfoResponse, DataItem, DataVersion, ExecuteMsg, InstantiateMsg,
    NumTokensResponse, OwnerOfResponse, QueryMsg, TokenInfoResponse,
};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    // Export core message schemas
    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);

    // Export response schemas
    export_schema(&schema_for!(OwnerOfResponse), &out_dir);
    export_schema(&schema_for!(TokenInfoResponse), &out_dir);
    export_schema(&schema_for!(NumTokensResponse), &out_dir);
    export_schema(&schema_for!(ContractInfoResponse), &out_dir);

    // Export data structure schemas
    export_schema(&schema_for!(DataItem), &out_dir);
    export_schema(&schema_for!(DataVersion), &out_dir);
    export_schema(&schema_for!(AccessLevel), &out_dir);
}
