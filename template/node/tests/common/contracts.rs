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

use crate::common::{consts::*, *};
use ep_eth::ContractInput;
use std::{
    io::{BufRead, BufReader},
    process,
};

/// Build contract
pub fn build(manifest_path: &str) -> process::Output {
    let manifest_arg = format!("--manifest-path={manifest_path}");

    let cmd_args = vec!["contract", "build", &manifest_arg, "--output-json"];

    process::Command::new("cargo")
        .args(cmd_args.as_slice())
        .output()
        .expect("failed to build with cargo-contract")
}

/// Deploy contract to the node exposed via `url`, and return the output
pub fn deploy(
    url: &str,
    manifest_path: &str,
    args: Vec<&str>,
    signer: Option<&str>,
) -> process::Output {
    let surl_arg = &format!("-s={}", signer.unwrap_or(ALITH_KEY));
    let manifest_arg = format!("--manifest-path={manifest_path}");
    let url_arg = format!("--url={}", url);
    let config_arg = &format!("--config=Ecdsachain");

    let mut cmd_args = vec![
        "contract",
        "instantiate",
        &surl_arg,
        &url_arg,
        &manifest_arg,
        "-x",
        "--skip-confirm",
        "--output-json",
        config_arg,
    ];

    let args = args
        .iter()
        .map(|a| format!("--args={a}"))
        .collect::<Vec<_>>();
    for s in &args {
        cmd_args.push(s)
    }

    process::Command::new("cargo")
        .args(cmd_args.as_slice())
        .output()
        .expect("failed to instantiate with cargo-contract")
}

/// Call contract deployed to env, and return the output
pub fn call(
    env: &Env<PolkadotConfig>,
    msg: &str,
    args: Vec<&str>,
    execute: bool,
    signer: Option<&str>,
) -> process::Output {
    let surl_arg = &format!("-s={}", signer.unwrap_or(ALITH_KEY));
    let manifest_arg = &format!("--manifest-path={}", env.contract_manifest_path());
    let url_arg = &format!("--url={}", env.ws_url());
    let contract_arg = &format!("--contract={}", env.contract_address());
    let msg_arg = &format!("--message={msg}");
    let config_arg = &format!("--config=Ecdsachain");

    let mut cmd_args = vec![
        "contract",
        "call",
        surl_arg,
        url_arg,
        manifest_arg,
        contract_arg,
        msg_arg,
        "--output-json",
        config_arg,
    ];

    let args = args
        .iter()
        .map(|a| format!("--args={a}"))
        .collect::<Vec<_>>();
    for s in &args {
        cmd_args.push(s)
    }

    if execute {
        cmd_args.push("-x")
    }

    let output = process::Command::new("cargo")
        .args(cmd_args.as_slice())
        .output()
        .expect("failed to call with cargo-contract");

    assert!(output.status.success(), "err: {:#?}", &output);

    output
}

/// Encode input data for contract call
pub fn encode(manifest_path: &str, msg: &str, args: Vec<&str>) -> ContractInput {
    let manifest_arg = &format!("--manifest-path={manifest_path}");
    let msg_arg = &format!("--message={msg}");

    let mut cmd_args = vec!["contract", "encode", manifest_arg, msg_arg];

    let args = args
        .iter()
        .map(|a| format!("--args={a}"))
        .collect::<Vec<_>>();
    for s in &args {
        cmd_args.push(s)
    }

    let output = process::Command::new("cargo")
        .args(cmd_args.as_slice())
        .output()
        .expect("failed to encode with cargo-contract");

    assert!(output.status.success(), "err: {:#?}", &output);

    // parse stdout for the encoded data string
    let bytes = BufReader::new(output.stdout.as_slice())
        .lines()
        .find_map(|line| {
            let line = line.expect("failed to get next line from cargo-contract stdout");
            line.split_once("Encoded data: ")
                .map(|(_, hex)| hex.to_owned())
        })
        .map(hex::decode)
        .expect("can't find encoded data string in cargo-contract stdout")
        .expect("can't deserialize encoded data from cargo-contract stdout");

    ContractInput::new(bytes)
}
