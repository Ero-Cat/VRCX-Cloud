use std::collections::HashMap;

use serde_json::Value;

use crate::common::ParamsBuilder;
use crate::ownership::OwnerRowId;

pub(super) fn scoped_params(owner_id: OwnerRowId) -> ParamsBuilder {
    ParamsBuilder::new().set("owner_id", owner_id)
}

pub(super) fn scoped_param_map(owner_id: OwnerRowId) -> HashMap<String, Value> {
    HashMap::from([("@owner_id".into(), Value::from(owner_id))])
}
