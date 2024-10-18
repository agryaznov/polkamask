use alloy::sol;
use serde::{Deserialize, Serialize};
// Codegen from ABI file to interact with the contract.
sol!(
    #[allow(clippy::too_many_arguments)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug, Serialize, Deserialize)]
    IERC20,
    "tests/abi/IERC20Minimal.json"
);
