#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(unexpected_cfgs)]

#[ink::contract(env = EthinkEnvironment)]
mod erc20 {
    use alloy_sol_types::{sol, SolType, sol_data::{String,Uint}};
    use ink::storage::Mapping;

    sol! {
        type Address is address;
        type Amount is uint128;
    }
    /// Custom environment with Ethereum-flavored Accountid
    #[derive(Clone)]
    pub struct EthinkEnvironment;

    impl ink_env::Environment for EthinkEnvironment {
        const MAX_EVENT_TOPICS: usize = 3;
        type AccountId = [u8; 20];
        type Balance = u128;
        type Hash = [u8; 32];
        type Timestamp = u64;
        type BlockNumber = u32;
        type ChainExtension = ::ink::env::NoChainExtension;
    }

    /// A simple ERC-20 contract working with ABI encoded i\o.
    #[ink(storage)]
    #[derive(Default)]
    pub struct Erc20 {
        /// Total token supply.
        total_supply: Balance,
        /// Mapping from owner to number of owned token.
        balances: Mapping<AccountId, Balance>,
        /// Mapping of the token amount which an account is allowed to withdraw
        /// from another account.
        allowances: Mapping<(AccountId, AccountId), Balance>,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    /// Event emitted when an approval occurs that `spender` is allowed to withdraw
    /// up to the amount of `value` tokens from `owner`.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    /// The ERC-20 error types.
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        /// Returned if not enough balance to fulfill a request is available.
        InsufficientBalance,
        /// Returned if not enough allowance to fulfill a request is available.
        InsufficientAllowance,
    }

    /// The ERC-20 result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl Erc20 {
        /// Creates a new ERC-20 contract with the specified initial supply.
        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {
            let mut balances = Mapping::default();
            let caller = Self::env().caller();
            balances.insert(caller, &total_supply);
            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: total_supply,
            });

            Self {
                total_supply,
                balances,
                allowances: Default::default(),
            }
        }

        /// Returns token decimals
        #[ink(message, selector = 0x313ce567)]
        pub fn decimals(&self) -> [u8;32] {
            Uint::<8>::abi_encode(&6u8).try_into().expect("ink: result value length is wrong")
        }

        /// Returns the total token supply.
        #[ink(message, selector = 0x18160ddd)]
        pub fn total_supply(&self) -> [u8;32] {
            Uint::<128>::abi_encode(&self.total_supply).try_into().expect("ink: result value length is wrong")
        }

        /// Returns the account balance for the specified `owner`
        /// NOTE: we can't use Vec<u8> as i\o type for contracts,
        /// because ink! will apply metadata [de|en]coding for its value as a custom type
        /// Hence we use fixed-sized byte arrays here as a workaround.
        /// Proper solution would be to upstream abi logic into ink!.
        /// Input len: 20 (accountId) + 12 (padding)
        /// Output len: 32 (U256)
        #[ink(message, selector = 0x70a08231)]
        pub fn balance_of(&self, input: [u8;32]) -> [u8;32] {
            let (owner,) = <(Address,)>::abi_decode_params(input.as_slice(), false).unwrap();
            Uint::<128>::abi_encode(&self.balance_of_internal(**owner)).try_into().expect("ink: result value length is wrong")
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        ///
        /// Returns `0` if no allowance has been set.
        ///
        /// Input len: 20 + 20 (accountId) + 12 (padding)
        /// Output len: 32 (U256)
        #[ink(message, selector = 0xdd62ed3e)]
        pub fn allowance(&self, input: [u8; 52]) -> [u8; 32] {
            let (owner, spender) =
                <(Address, Address)>::abi_decode_params(input.as_slice(), false).unwrap();
            Uint::<128>::abi_encode(&self.allowance_internal(**owner, **spender)).try_into().expect("ink: result value length is wrong")
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// Input len: 20 (accountId) + 32 (U256) + 12 (padding)
        #[ink(message, selector = 0xa9059cbb)]
        pub fn transfer(&mut self, input: [u8; 64]) -> [u8;32] {
            let from = self.env().caller();
            ink::env::debug_println!("Hello from contract:transfer(). Input is: {:x?}", &input);
            let (to, value) =
                <(Address, Amount)>::abi_decode_params(input.as_slice(), false).unwrap();
            self.transfer_from_to(&from, &to, value).expect("ink: insufficient allowance or balance");
            Uint::<8>::abi_encode(&(true as u8)).try_into().expect("ink: result value length is wrong")
        }

        /// Allows `spender` to withdraw from the caller's account multiple times, up to
        /// the `value` amount.
        ///
        /// If this function is called again it overwrites the current allowance with
        /// `value`.
        ///
        /// An `Approval` event is emitted.
        ///
        /// Input len: 20 (accountId) + 16 (U256) + 12 (padding)
        #[ink(message, selector = 0x095ea7b3)]
        pub fn approve(&mut self, input: [u8; 48]) -> Result<()> {
            let owner = self.env().caller();
            let (spender, value) =
                <(Address, Amount)>::abi_decode_params(input.as_slice(), false).unwrap();
            self.allowances.insert((&owner, **spender), &value);
            self.env().emit_event(Approval {
                owner,
                spender: **spender,
                value,
            });
            Ok(())
        }

        /// Transfers `value` tokens on the behalf of `from` to the account `to`.
        ///
        /// This can be used to allow a contract to transfer tokens on ones behalf and/or
        /// to charge fees in sub-currencies, for example.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientAllowance` error if there are not enough tokens allowed
        /// for the caller to withdraw from `from`.
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the account balance of `from`.
        ///
        /// Input len: 20 + 20 (accountId) + 16 (U256) + 12 (padding)
        #[ink(message, selector = 0x23b872dd)]
        pub fn transfer_from(&mut self, input: [u8; 68]) -> Result<()> {
            let caller = self.env().caller();
            let (from, to, value) =
                <(Address, Address, Amount)>::abi_decode_params(input.as_slice(), false).unwrap();
            let allowance = self.allowance_internal(**from, caller);
            if allowance < value {
                return Err(Error::InsufficientAllowance);
            }
            self.transfer_from_to(&from, &to, value)?;
            // We checked that allowance >= value
            #[allow(clippy::arithmetic_side_effects)]
            self.allowances
                .insert((**from, &caller), &(allowance - value));
            Ok(())
        }

        // TODO doc
        // TODO output size reasoning
        #[ink(message, selector = 0x06fdde03)]
        pub fn name(&self) -> [u8;96] {
            String::abi_encode("ink! token").try_into().expect("ink: result value length is wrong")
        }

        // TODO doc
        // TODO output size reasoning
        #[ink(message, selector = 0x95d89b41)]
        pub fn symbol(&self) -> [u8;96] {
            String::abi_encode("ink!").try_into().expect("ink: result value length is wrong")
        }
        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the caller's account balance.
        fn transfer_from_to(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            value: Balance,
        ) -> Result<()> {
            let from_balance = self.balance_of_internal(*from);
            ink::env::debug_println!("Hello from contract:transfer_from_to(). from_balance is: {:?}", &from_balance);
            if from_balance < value {
                return Err(Error::InsufficientBalance);
            }
            // We checked that from_balance >= value
            #[allow(clippy::arithmetic_side_effects)]
            self.balances.insert(from, &(from_balance - value));
            let to_balance = self.balance_of_internal(*to);
            self.balances
                .insert(to, &(to_balance.checked_add(value).unwrap()));
            self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                value,
            });
            Ok(())
        }

        fn balance_of_internal(&self, owner: AccountId) -> Balance {
            self.balances.get(owner).unwrap_or_default()
        }

        fn allowance_internal(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowances.get((&owner, &spender)).unwrap_or_default()
        }
    }
}
