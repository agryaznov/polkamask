// SPDX-License-Identifier: Apache-2.0
//
// This file is part of Ethink.
//
// Copyright (c) 2023-2024 Alexander Gryaznov.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Integraion tests for ethink! ERC20
#![allow(non_snake_case)]
use alloy::{
    primitives::{Address, U256},
    providers::{Provider, ProviderBuilder},
};
use serde_json::Deserializer;
use sp_core::{ecdsa, Pair};
use std::sync::Once;

use common::{codegen::*, consts::*, *};
use ep_eth::{AccountId20, EnvelopedEncodable, EthTxInput, TransactionAction};
use ep_mapping::SubstrateWeight;
use ethink_runtime::ED;

mod common;

const ERC20_PATH: &'static str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../dapp/contracts/erc20abi.ink/Cargo.toml"
);
// Sync primitive to build contract only once per test suite run
static ONCE: Once = Once::new();

#[tokio::test]
async fn transfer_works() {
    // ERC20 is being deployed with 1230 *10^6 supply
    const ERC20_SUPPLY: u128 = 1_230_000_000;

    // SUBSTRATE RPC: Spawn node and deploy contract
    let mut env: Env<PolkadotConfig> = prepare_node_and_contract!(
        ONCE,
        ERC20_PATH,
        vec![&ERC20_SUPPLY.to_string()],
        BALTATHAR_KEY
    );

    // TODO put to Env
    // TODO AccountId20 -> Address
    let contract_addr = Address::from(env.contract_address().0);
    // Build alloy ETH RPC provider
    let rpc = ProviderBuilder::new().on_http(
        env.http_url()
            .parse()
            .expect("failed to build alloy provider"),
    );
    // ETH RPC: query contract balance
    let contract_bal = rpc
        .get_balance(contract_addr)
        .await
        .expect("can't get balance");
    // Deployed contract should have ED balance
    assert_eq!(contract_bal, U256::from(ED));
    // Get our ink! contract instance as Solidity contract
    let contract = IERC20::new(contract_addr, rpc);
    // ETH RPC: query ERC20 token balances
    let (cal_a, cal_b) = (
        contract.balanceOf(ALITH_ADDR),
        contract.balanceOf(BALTATHAR_ADDR),
    );
    let (a_bal, b_bal) = (
        cal_a.call().await.unwrap()._0,
        cal_b.call().await.unwrap()._0,
    );
    // Alith ERC20 token balance should be zero
    assert_eq!(a_bal, U256::ZERO);
    // Baltathar ERC20 token balance should be total supply
    assert_eq!(b_bal, U256::from(ERC20_SUPPLY));

    // ETH RPC: send tx to transfer 100k of ERC20 to Alith
    let _tx_hash = contract
        .transfer(ALITH_ADDR, U256::from(100_000))
        .from(BALTATHAR_ADDR)
        .gas(u64::MAX)
        .send()
        .await
        .unwrap();
    // TODO: make rpc return proper receipt, and wait for tx execution completeness like this:
    //        .watch()
    //        .await
    //        .unwrap();

    // Wait tx to be included into block
    let _ = &env.wait_for_event("Ethink.TxExecuted", 3).await;
    // ETH RPC: query ERC20 token balances
    let (a_bal, b_bal) = (
        cal_a.call().await.unwrap()._0,
        cal_b.call().await.unwrap()._0,
    );
    // Alith ERC20 token balance should become 100k
    assert_eq!(a_bal, U256::from(100_000));
    // Baltathar ERC20 token balance should be total_supply - 100k
    assert_eq!(b_bal, U256::from(ERC20_SUPPLY - 100_000));
}

// TODO
#[tokio::test]
#[ignore]
async fn allowances_work() {
    // Spawn node and deploy contract
    let mut env: Env<PolkadotConfig> =
        prepare_node_and_contract!(ONCE, ERC20_PATH, vec!["10_000"], BALTATHAR_KEY);
    // (ERC20 is deployed with 10_000 supply)
    // Make ETH RPC request (to unauthorized transfer 2_000 to Alith)
    let input = EthTxInput {
        signer: ecdsa::Pair::from_string(ALITH_KEY, None).unwrap(),
        action: TransactionAction::Call(env.contract_address().into()),
        data: encode!(
            ERC20_PATH,
            "transfer_from",
            vec![BALTATHAR_ADDRESS, ALITH_ADDRESS, "2_000"]
        ),
        ..Default::default()
    };
    let tx = ep_eth::compose_and_sign_tx(input);
    let tx_hex = format!("0x{:x}", &tx.clone().encode());
    let rs = rpc_rq!(env,
    {
      "jsonrpc": "2.0",
      "method": "eth_sendRawTransaction",
      "params": [ &tx_hex ],
      "id": 0
     });
    // Handle response
    let json = to_json_val!(rs);
    ensure_no_err!(&json);
    let _tx_hash = extract_result!(&json);
    // Wait until tx gets executed
    let _ = &env.wait_for_event("ethink.EthTransactionExecuted", 2).await;
    // Check state
    let output = call!(env, "balance_of", vec![ALITH_ADDRESS]);
    let rs = Deserializer::from_slice(&output.stdout);
    // Alith balance should stay 0
    assert_eq!(
        json_get!(rs["data"]["Tuple"]["values"][0]["UInt"])
            .as_number()
            .expect("can't parse cargo contract output")
            .as_u64(),
        Some(0u64)
    );
    // Make ETH RPC request (to approve transfer 2_000 to Alith)
    let call_data = encode!(ERC20_PATH, "approve", vec![ALITH_ADDRESS, "2_000"]);
    let rs = rpc_rq!(env,
    {
      "jsonrpc": "2.0",
      "method": "eth_sendTransaction",
      "params": [{
                  "from": BALTATHAR_ADDRESS,
                  "to": &env.contract_address(),
                  "data": call_data,
                  "gas": SubstrateWeight::max()
                 },
                 "latest"],
      "id": 1
    });
    // Handle response
    let json = to_json_val!(rs);
    ensure_no_err!(&json);
    let _tx_hash = extract_result!(&json);
    // Wait until tx gets executed
    let _ = &env.wait_for_event("ethink.EthTransactionExecuted", 3).await;
    // Make ETH RPC request (to authorized transfer 2_000 to Alith)
    let tx_hex = format!("0x{:x}", &tx.encode());
    let rs = rpc_rq!(env,
    {
      "jsonrpc": "2.0",
      "method": "eth_sendRawTransaction",
      "params": [ &tx_hex ],
      "id": 2
     });
    // Handle response
    let json = to_json_val!(rs);
    ensure_no_err!(&json);
    let _tx_hash = extract_result!(&json);
    // Wait until tx gets executed
    let _ = &env.wait_for_event("ethink.EthTransactionExecuted", 4).await;
    // Check state
    let output = call!(env, "balance_of", vec![ALITH_ADDRESS]);
    let rs = Deserializer::from_slice(&output.stdout);
    // Alith balance should become 2_000
    assert_eq!(
        json_get!(rs["data"]["Tuple"]["values"][0]["UInt"])
            .as_number()
            .expect("can't parse cargo contract output")
            .as_u64(),
        Some(2_000u64)
    );
}
