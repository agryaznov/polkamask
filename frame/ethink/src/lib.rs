// SPDX-License-Identifier: Apache-2.0
//
// This file was derived from Frontier (pallet-ethereum),
// and modified to become part of Ethink.
//
// Copyright (c) (Frontier): 2020-2022 Parity Technologies (UK) Ltd.
// Copyright (c) (Ethink):   2023-2024 Alexander Gryaznov.
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

//! # ethink pallet
//!
//! The ethink pallet works as a proxy in front of `pallet_contracts` for the transactions
//! coming from the Ethereum RPC.

// `no_std` when compiling to WebAssembly
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::comparison_chain, clippy::large_enum_variant)]
use frame_support::{
    dispatch::{DispatchInfo, PostDispatchInfo},
    traits::fungible::{Inspect, Mutate},
    weights::Weight,
};
use frame_system::{pallet_prelude::OriginFor, CheckWeight, Pallet as System};
use scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::{H160, H256, U256};
use sp_runtime::{
    traits::{Block as BlockT, DispatchInfoOf, Dispatchable},
    transaction_validity::{
        InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransactionBuilder,
    },
    DispatchError, RuntimeDebug,
};
use sp_std::vec::Vec;
use sp_std::{marker::PhantomData, prelude::*};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod exec;

pub mod weights;

#[cfg(all(feature = "std", test))]
mod mock;
#[cfg(all(feature = "std", test))]
mod tests;

pub use self::{pallet::*, weights::WeightInfo};
pub use ep_eth::{EthTransaction, LegacyTransactionMessage, Receipt, TransactionAction};
pub use exec::Executor;

pub type BalanceOf<T> =
    <<T as Config>::Currency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

pub const ETH_BASE_GAS_FEE: u64 = 21_000;

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum RawOrigin {
    EthTransaction(H160),
}

impl<A: From<H160>> From<RawOrigin> for frame_system::RawOrigin<A> {
    fn from(s: RawOrigin) -> Self {
        let RawOrigin::EthTransaction(acc) = s;
        // Signature was already checked upon checking UncheckedExtrinsic, via check_self_contained()
        frame_system::RawOrigin::<A>::Signed(acc.into())
    }
}

pub fn ensure_eth_transaction<OuterOrigin>(o: OuterOrigin) -> Result<RawOrigin, &'static str>
where
    OuterOrigin: Into<Result<RawOrigin, OuterOrigin>>,
{
    o.into()
        .map_err(|_| "Bad origin: not a valid Ethereum transaction")
}

impl<T> Call<T>
where
    OriginFor<T>: Into<Result<RawOrigin, OriginFor<T>>>,
    T: Send + Sync + Config,
    T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    T::AccountId: From<sp_core::H160> + Into<sp_core::H160> + AsRef<[u8]>,
    T::Contracts: Executor<T>,
    BalanceOf<T>: TryFrom<sp_core::U256>,
{
    pub fn is_self_contained(&self) -> bool {
        matches!(self, Call::transact { .. })
    }

    pub fn check_self_contained(&self) -> Option<Result<H160, TransactionValidityError>> {
        match self {
            Call::transact { tx } => Some(Pallet::<T>::check_eth_signature(tx)),
            // Not a self-contained call
            _ => None,
        }
    }

    pub fn pre_dispatch_self_contained(
        &self,
        _origin: &H160,
        _dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
        _len: usize,
    ) -> Option<Result<(), TransactionValidityError>> {
        Some(Ok(()))
    }

    pub fn validate_self_contained(
        &self,
        origin: &H160,
        dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
        len: usize,
    ) -> Option<TransactionValidity> {
        match &self {
            Call::transact { tx } => {
                if let Err(e) = CheckWeight::<T>::do_validate(dispatch_info, len) {
                    return Some(Err(e));
                }
                let tx_nonce = match tx {
                    EthTransaction::Legacy(t) => t.nonce,
                    EthTransaction::EIP2930(t) => t.nonce,
                    EthTransaction::EIP1559(t) => t.nonce,
                };
                let builder = ValidTransactionBuilder::default().and_provides((origin, tx_nonce));

                Some(builder.build())
            }
            _ => None,
        }
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::origin]
    pub type Origin = RawOrigin;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// The overarching call type.
        type Call: Dispatchable<RuntimeOrigin = Self::RuntimeOrigin, PostInfo = PostDispatchInfo>;
        /// The fungible in which fees are paid and contract balances are held.
        type Currency: Inspect<Self::AccountId> + Mutate<Self::AccountId>;
        /// Contracts engine
        type Contracts: Executor<Self>;
        /// Weights for extrinsics
        type WeightInfo: WeightInfo;
    }

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        OriginFor<T>: Into<Result<RawOrigin, OriginFor<T>>>,
        T::AccountId: From<sp_core::H160> + Into<sp_core::H160> + AsRef<[u8]>,
        T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
        T::Contracts: Executor<T>,
        BalanceOf<T>: TryFrom<sp_core::U256>,
    {
        /// Transact a call coming from Ethereum RPC
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::transact())]
        pub fn transact(origin: OriginFor<T>, tx: EthTransaction) -> DispatchResult {
            let origin: frame_system::RawOrigin<T::AccountId> =
                ensure_eth_transaction(origin)?.into();
            // We received Ethereum transaction,
            // need to route it either as a contract call or just a balance transfer
            let from = origin.clone();
            let from = from.as_signed().ok_or(Error::<T>::BadEthSignature)?;
            let (to, value, data, gas_limit) =
                Self::unpack_eth_tx(&tx).ok_or(Error::<T>::TxNotSupported)?;
            // CREATE is not supported
            let to = to.ok_or(Error::<T>::TxNotSupported)?;
            // Increment nonce of the sender account
            System::<T>::inc_account_nonce(from);
            // Compose proper destination pallet call
            let call = T::Contracts::build_call(to.clone(), value, data.clone(), gas_limit)
                .ok_or(Error::<T>::TxNotSupported)?;
            // Make call
            log::debug!(target: "ethink:pallet", "Dispatching CALL {:?}\n DATA in hex: {}", &call, hex::encode(&data));
            let _ = call.dispatch(origin.into()).map_err(|e| {
                log::error!(target: "ethink:pallet", "Failed: {:?}", &e);
                Error::<T>::TxExecutionFailed
            })?;
            // Deposit Event
            let tx_hash = tx.hash();
            let from = (*from).clone().into();
            Self::deposit_event(Event::TxExecuted {
                from,
                to: to.into(),
                tx_hash,
            });

            Ok(())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event {
        /// A call coming from ETH RPC was successfully executed.
        TxExecuted { from: H160, to: H160, tx_hash: H256 },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Signature is invalid
        BadEthSignature,
        /// Type of transaction is not supported
        TxNotSupported,
        /// Transaction execution failed
        TxExecutionFailed,
    }

    /// The current Ethereum receipts.
    #[pallet::storage]
    pub type CurrentReceipts<T: Config> = StorageValue<_, Vec<Receipt>>;
}

impl<T> Pallet<T>
where
    T: Config,
    T::AccountId: AsRef<[u8]>,
    T::Contracts: Executor<T>,
{
    pub fn code_at(address: T::AccountId) -> Option<Vec<u8>> {
        T::Contracts::code_at(&address)
    }

    pub fn contract_call(
        from: T::AccountId,
        to: T::AccountId,
        data: Vec<u8>,
        value: BalanceOf<T>,
        gas_limit: Weight,
    ) -> <T::Contracts as Executor<T>>::ExecResult {
        log::error!(target: "ethink:pallet", "Contract: {:?} call with input: {}", hex::encode(&to), hex::encode(&data));
        T::Contracts::call(from, to, data, value, gas_limit)
    }

    pub fn gas_estimate(
        from: T::AccountId,
        to: T::AccountId,
        data: Vec<u8>,
        value: BalanceOf<T>,
        gas_limit: Weight,
    ) -> Result<U256, DispatchError> {
        T::Contracts::gas_estimate(from, to, data, value, gas_limit)
    }

    fn check_eth_signature(tx: &EthTransaction) -> Result<H160, TransactionValidityError> {
        let mut sig = [0u8; 65];
        let mut msg = [0u8; 32];
        match tx {
            EthTransaction::Legacy(t) => {
                sig[0..32].copy_from_slice(&t.signature.r()[..]);
                sig[32..64].copy_from_slice(&t.signature.s()[..]);
                sig[64] = t.signature.standard_v();
                msg.copy_from_slice(&LegacyTransactionMessage::from(t.clone()).hash()[..]);
            }
            // We only support Legacy. EIP2930 and EIP1559 are not supported
            _ => {
                return Err(TransactionValidityError::Invalid(
                    InvalidTransaction::BadProof,
                ))
            }
        }
        // We check ethereum signature here, and derive sender account from it.
        sp_io::crypto::secp256k1_ecdsa_recover(&sig, &msg)
            .map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::BadProof))
            .map(|p| H160::from(H256::from(sp_io::hashing::keccak_256(&p))))
    }

    fn unpack_eth_tx(tx: &EthTransaction) -> Option<(Option<T::AccountId>, U256, Vec<u8>, U256)>
    where
        <T as frame_system::Config>::AccountId: From<ep_eth::H160>,
    {
        match tx {
            EthTransaction::Legacy(t) => Some((
                match t.action {
                    TransactionAction::Call(h) => Some(h.into()),
                    TransactionAction::Create => None,
                },
                t.value,
                t.input.clone(),
                t.gas_limit,
            )),
            // We only support Legacy, EIP2930 and EIP1559 are not supported
            _ => None,
        }
    }
}

#[derive(Eq, PartialEq, Clone, RuntimeDebug)]
pub enum ReturnValue {
    Bytes(Vec<u8>),
    Hash(H160),
}

sp_api::decl_runtime_apis! {
    /// Runtime-exposed API necessary for ETH-compatibility layer.
    pub trait EthinkAPI {
        /// Return contract's code hash
        fn code_at(address: H160) -> Option<Vec<u8>>;

        /// Return runtime-defined CHAIN_ID.
        fn chain_id() -> u64;

        /// Return account balance.
        fn account_free_balance(address: H160) -> U256;

        /// Return account nonce.
        fn nonce(address: H160) -> U256;

        /// Call contract (without extrinsic submission)
        fn call(
            from: H160,
            to: H160,
            data: Vec<u8>,
            value: u128,
            gas_limit: Weight,
        ) -> Result<Vec<u8>, sp_runtime::DispatchError>;

        /// Estimate gas needed for a contract call
        fn gas_estimate(
            from: H160,
            to: H160,
            data: Vec<u8>,
            value: u128,
            gas_limit: Weight,
        ) -> Result<U256, sp_runtime::DispatchError>;

        /// Wrap Ethereum transaction into an extrinsic
        fn build_extrinsic(from: EthTransaction) -> <Block as BlockT>::Extrinsic;
    }
}
