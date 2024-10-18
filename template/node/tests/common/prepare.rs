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
//! Prelude actions performed in most of the tests
use crate::common::{consts::*, node::*, *};
use serde_json::Deserializer;
use std::{str::FromStr, sync::Once};

/// Spawn a node and deploy a contract to it
pub async fn node_and_contract<R: subxt::Config>(
    once: &Once,
    manifest_path: &str,
    constructor_args: Vec<&str>,
    signer: Option<&str>,
) -> Env<R> {
    // Build contract (this is done just done once per test suite run)
    once.call_once(|| {
        contracts::build(manifest_path);
    });

    let node = spawn_node(signer).await;
    // deploy contract
    let output = contracts::deploy(
        node.url(Protocol::WS).as_str(),
        manifest_path,
        constructor_args,
        signer,
    );

    if !output.status.success() {
        use std::io::Write;
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
        panic!("'cargo contract instantantiate' failed")
    }
    // Look for contract address in the json output
    let rs = Deserializer::from_slice(&output.stdout);
    let address = json_get!(rs["contract"])
        .as_str()
        .map(AccountId20::from_str)
        .expect("contract address not returned")
        .expect("can't decode contract address");

    let manifest_path = manifest_path.to_string();
    let contract = Contract {
        manifest_path,
        address,
    };

    Env::new(node, Some(contract))
}

pub async fn node<R: subxt::Config>(signer: Option<&str>) -> Env<R> {
    let node = spawn_node(signer).await;

    Env::new(node, None)
}

async fn spawn_node<R: subxt::Config>(signer: Option<&str>) -> TestNodeProcess<R> {
    let mut builder = TestNodeProcess::<R>::build(NODE_BIN);
    if let Some(key) = signer {
        builder.with_signer(key)
    } else {
        &mut builder
    }
    .spawn()
    .await
    .unwrap_or_else(|err| ::core::panic!("Error spawning ethink-node: {:?}", err))
}
