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
pub mod node;
#[macro_use]
pub mod macros;
pub mod codegen;
pub mod contracts;
pub mod prepare;

pub mod consts {
    use alloy::primitives::{address, Address};
    // Well-known accounts taken from Moonbeam
    pub const NODE_BIN: &'static str = env!("CARGO_BIN_EXE_ethink-node");
    // TODO remove strs
    pub const ALITH_ADDRESS: &'static str = "0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac";
    pub const ALITH_ADDR: Address = address!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac");
    pub const ALITH_KEY: &'static str =
        "0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133";
    pub const BALTATHAR_ADDRESS: &'static str = "0x3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0";
    pub const BALTATHAR_ADDR: Address = address!("3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0");
    pub const BALTATHAR_KEY: &'static str =
        "0x8075991ce870b93a8870eca0c0f91913d12f47948ca0fd25b49c6fa7cdbeee8b";
}

use crate::AccountId20;
use futures::StreamExt;
use node::{Protocol, TestNodeProcess};

#[derive(Clone)]
pub struct Contract {
    pub manifest_path: String,
    pub address: AccountId20,
}

// Testing environment, consisting of a node with a possibly deployed contract
pub struct Env<R: subxt::Config> {
    pub node: TestNodeProcess<R>,
    contract: Option<Contract>,
}

impl<R: subxt::Config> Env<R> {
    pub fn new(node: TestNodeProcess<R>, contract: Option<Contract>) -> Self {
        Env { node, contract }
    }

    pub fn contract_address(&self) -> AccountId20 {
        self.contract
            .as_ref()
            .expect("env does not have the contract deployed!")
            .address
    }

    pub fn contract_manifest_path(&self) -> String {
        self.contract
            .as_ref()
            .expect("env does not have the contract deployed!")
            .manifest_path
            .to_owned()
    }

    pub fn ws_url(&self) -> String {
        self.node.url(Protocol::WS)
    }

    pub fn http_url(&self) -> String {
        self.node.url(Protocol::HTTP)
    }

    /// Wait until a specified event is emitted in a finalized block,
    /// but no longer than `timeout` number of blocks.
    pub async fn wait_for_event(&mut self, fullname: &str, timeout: usize) {
        if let Some((pallet, variant)) = fullname.rsplit_once(".") {
            let blocks_sub = &mut self
                .node
                .client()
                .blocks()
                .subscribe_finalized()
                .await
                .expect("can't subscribe to finalized blocks")
                .take(timeout);

            while let Some(block) = blocks_sub.next().await {
                let block = block.expect("can't get next finalized block");
                let events = block.events().await.expect("can't get events from block");
                if let Some(_) = events.iter().find(|e| {
                    let event = e.as_ref().expect("failed to read event");
                    event.pallet_name().eq(pallet) && event.variant_name().eq(variant)
                }) {
                    break;
                }
            }
        }
    }

    /// Wait until a block with the given number gets finalized,
    pub async fn wait_for_block_number(&mut self, number: u64) {
        let blocks_sub = &mut self
            .node
            .client()
            .blocks()
            .subscribe_finalized()
            .await
            .expect("can't subscribe to finalized blocks");

        while let Some(block) = blocks_sub.next().await {
            let block = block.expect("can't get next finalized block");
            if block.number().into() == number {
                break;
            }
        }
    }
}

pub enum PolkadotConfig {}

impl subxt::Config for PolkadotConfig {
    type Hash = sp_core::H256;
    type Hasher = subxt::config::substrate::BlakeTwo256;
    type AccountId = subxt::config::substrate::AccountId32;
    type Address = sp_runtime::MultiAddress<Self::AccountId, u32>;
    type Header =
        subxt::config::substrate::SubstrateHeader<u32, subxt::config::substrate::BlakeTwo256>;
    type Signature = sp_runtime::MultiSignature;
    type ExtrinsicParams = subxt::config::substrate::SubstrateExtrinsicParams<Self>;
    type AssetId = u32;
}
