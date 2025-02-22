
//! Autogenerated weights for `pallet_ethink`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 32.0.0
//! DATE: 2025-01-23, STEPS: `100`, REPEAT: `100`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `station`, CPU: `AMD Ryzen Threadripper PRO 5945WX 12-Cores`
//! WASM-EXECUTION: `Compiled`, CHAIN: `None`, DB CACHE: 1024

// Executed Command:
// ./target/release/ethink-node
// benchmark
// pallet
// --pallet=pallet_ethink
// --extrinsic=*
// --wasm-execution=compiled
// --repeat=100
// --steps=100
// --output=src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

pub trait WeightInfo {
	fn transact() -> Weight;
}

/// Weight functions for `pallet_ethink`.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `System::Account` (r:2 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(116), added: 2591, mode: `MaxEncodedLen`)
	/// Storage: `Contracts::ContractInfoOf` (r:1 w:0)
	/// Proof: `Contracts::ContractInfoOf` (`max_values`: None, `max_size`: Some(1783), added: 4258, mode: `MaxEncodedLen`)
	/// Storage: `System::Number` (r:1 w:0)
	/// Proof: `System::Number` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `System::ExecutionPhase` (r:1 w:0)
	/// Proof: `System::ExecutionPhase` (`max_values`: Some(1), `max_size`: Some(5), added: 500, mode: `MaxEncodedLen`)
	/// Storage: `System::EventCount` (r:1 w:1)
	/// Proof: `System::EventCount` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `System::Events` (r:1 w:1)
	/// Proof: `System::Events` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn transact() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `129`
		//  Estimated: `6172`
		// Minimum execution time: 31_719_000 picoseconds.
		Weight::from_parts(32_651_000, 0)
			.saturating_add(Weight::from_parts(0, 6172))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(3))
	}
}
