#![cfg(feature = "runtime-benchmarks")]

use crate::{
    BalanceOf, Call, Config, DispatchInfo, Dispatchable, EthTransaction, OriginFor, Pallet,
    PostDispatchInfo, RawOrigin, U256,
};
use ep_eth::{
    AccountId20, EthereumSignature, EthereumSigner, LegacyTransaction, LegacyTransactionMessage,
    Receipt, TransactionAction, TransactionSignature,
};
use frame_benchmarking::v2::*;
use hex_literal::hex;
use sp_core::Pair;
use sp_std::vec;

const ALITH: [u8; 20] = [
    242, 79, 243, 169, 207, 4, 199, 29, 188, 148, 208, 181, 102, 247, 162, 123, 148, 86, 108, 172,
];

#[benchmarks(
    where
     T: Config,
     T::AccountId: From<sp_core::H160> + AsRef<[u8]> + Into<sp_core::H160>,
     T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
     OriginFor<T>: Into<Result<RawOrigin, OriginFor<T>>>,
     BalanceOf<T>: TryFrom<sp_core::U256>,
     frame_system::RawOrigin<T::AccountId>:  From<RawOrigin>,
)]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn transact() -> Result<(), BenchmarkError> {
        // dumb contract code with size of
        // MaxCodeLen = ConstU32<{ 256 * 1024 }>
        // (as configured in runtime)
        let code = vec![42u8; 256 * 1024];
        // Compose transaction
        let signer = sp_core::ecdsa::Pair::generate().0; //EthereumSigner = ALITH.into();
                                                         // TODO make ep_eth helpers for this usable from here (needs runtime-benchmarks feat)
        let msg = LegacyTransactionMessage {
            action: TransactionAction::Create,
            input: code.into(),
            nonce: 0u8.into(),
            gas_price: 0u8.into(),
            gas_limit: U256::MAX,
            value: 0u8.into(),
            chain_id: None,
        };
        let sig = EthereumSignature::new(signer.into());
        let sig: Option<TransactionSignature> = sig.into();
        let signature = sig.expect("signer generated no signature");

        let tx = EthTransaction::Legacy(LegacyTransaction {
            nonce: msg.nonce,
            gas_price: msg.gas_price,
            gas_limit: msg.gas_limit,
            action: msg.action,
            value: msg.value,
            input: msg.input,
            signature,
        });

        #[extrinsic_call]
        _(RawOrigin::EthTransaction(ALITH.into()), tx);

        Ok(())
    }
}
