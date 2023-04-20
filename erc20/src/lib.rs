//! A library for developing ERC20 tokens for the Casper network.
//!
//! The main functionality is provided via the [`ERC20`] struct, and is intended to be consumed by a
//! smart contract written to be deployed on the Casper network.
//!
//! To create an example ERC20 contract which uses this library, use the cargo-casper tool:
//!
//! ```bash
//! cargo install cargo-casper
//! cargo casper --erc20 <PATH TO NEW PROJECT>
//! ```

#![warn(missing_docs)]

extern crate alloc;

mod address;
mod allowances;
mod balances;
pub mod constants;
mod detail;
mod dict;
pub mod entry_points;
mod error;
mod event;
mod mintids;
mod request_map;

use alloc::string::{String, ToString};

use once_cell::unsync::OnceCell;

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{account::AccountHash, contracts::NamedKeys, EntryPoints, Key, URef, U256};

pub use address::Address;
use constants::{
    ALLOWANCES_KEY_NAME, BALANCES_KEY_NAME, DECIMALS_KEY_NAME, ERC20_TOKEN_CONTRACT_KEY_NAME,
    MINTER_KEY_NAME, NAME_KEY_NAME, SYMBOL_KEY_NAME, TOTAL_SUPPLY_KEY_NAME,
};
pub use error::Error;

/// Implementation of ERC20 standard functionality.
#[derive(Default)]
pub struct ERC20 {
    balances_uref: OnceCell<URef>,
    allowances_uref: OnceCell<URef>,
    total_supply_uref: OnceCell<URef>,
    minter_uref: OnceCell<URef>,
    swap_fee_uref: OnceCell<URef>,
    dev_uref: OnceCell<URef>,
    mintids_uref: OnceCell<URef>,
    requestid_uref: OnceCell<URef>,
    request_map_uref: OnceCell<URef>,
}

impl ERC20 {
    fn new(
        balances_uref: URef,
        allowances_uref: URef,
        total_supply_uref: URef,
        minter_uref: URef,
        swap_fee_uref: URef,
        dev_uref: URef,
        mintids_uref: URef,
        requestid_uref: URef,
        request_map_uref: URef,
    ) -> Self {
        Self {
            balances_uref: balances_uref.into(),
            allowances_uref: allowances_uref.into(),
            total_supply_uref: total_supply_uref.into(),
            minter_uref: minter_uref.into(),
            swap_fee_uref: swap_fee_uref.into(),
            dev_uref: dev_uref.into(),
            mintids_uref: mintids_uref.into(),
            requestid_uref: requestid_uref.into(),
            request_map_uref: request_map_uref.into(),
        }
    }

    fn total_supply_uref(&self) -> URef {
        *self
            .total_supply_uref
            .get_or_init(dict::get_uref_total_supply)
    }

    fn swap_fee_uref(&self) -> URef {
        *self.swap_fee_uref.get_or_init(dict::get_uref_swap_fee)
    }

    fn dev_uref(&self) -> URef {
        *self.dev_uref.get_or_init(dict::get_uref_dev)
    }

    fn minter_uref(&self) -> URef {
        *self.minter_uref.get_or_init(dict::get_uref_minter)
    }

    fn read_total_supply(&self) -> U256 {
        dict::read_total_supply_from(self.total_supply_uref())
    }

    fn write_total_supply(&self, total_supply: U256) {
        dict::write_total_supply_to(self.total_supply_uref(), total_supply)
    }

    fn read_swap_fee(&self) -> U256 {
        dict::read_swap_fee_from(self.swap_fee_uref())
    }

    fn write_swap_fee(&self, swap_fee: U256) {
        dict::write_swap_fee_to(self.swap_fee_uref(), swap_fee)
    }

    fn read_dev(&self) -> String {
        dict::read_dev_from(self.dev_uref())
    }

    fn write_dev(&self, dev: String) {
        dict::write_dev_to(self.dev_uref(), dev)
    }

    fn read_minter(&self) -> String {
        dict::read_minter_from(self.minter_uref())
    }

    fn write_minter(&self, minter: String) {
        dict::write_minter_to(self.minter_uref(), minter)
    }

    fn balances_uref(&self) -> URef {
        *self.balances_uref.get_or_init(balances::get_balances_uref)
    }

    fn read_balance(&self, owner: Address) -> U256 {
        balances::read_balance_from(self.balances_uref(), owner)
    }

    fn write_balance(&mut self, owner: Address, amount: U256) {
        balances::write_balance_to(self.balances_uref(), owner, amount)
    }

    fn request_map_uref(&self) -> URef {
        *self
            .request_map_uref
            .get_or_init(request_map::request_map_uref)
    }
    /// Returns read_request_mapn.
    pub fn read_request_map(&self, unique_id: &String) -> U256 {
        request_map::read_request_map_from(self.request_map_uref(), unique_id)
    }

    fn write_request_map(&mut self, unique_id: &String, index: U256) {
        request_map::write_request_map_to(self.request_map_uref(), unique_id, index)
    }

    fn allowances_uref(&self) -> URef {
        *self
            .allowances_uref
            .get_or_init(allowances::allowances_uref)
    }

    fn mintids_uref(&self) -> URef {
        *self.mintids_uref.get_or_init(mintids::mintids_uref)
    }

    fn requestid_uref(&self) -> URef {
        *self.requestid_uref.get_or_init(dict::get_uref_requestid)
    }

    fn read_allowance(&self, owner: Address, spender: Address) -> U256 {
        allowances::read_allowance_from(self.allowances_uref(), owner, spender)
    }

    fn write_allowance(&mut self, owner: Address, spender: Address, amount: U256) {
        allowances::write_allowance_to(self.allowances_uref(), owner, spender, amount)
    }
    /// Returns the mintid whether mint or not.
    pub fn read_mintid(&self, mintid: &String) -> u64 {
        mintids::read_mintid_from(self.mintids_uref(), mintid)
    }

    fn write_mintid(&mut self, mintid: &String) {
        mintids::write_mintid_to(self.mintids_uref(), mintid)
    }
    /// Returns the current request id.
    pub fn read_requestid(&self) -> U256 {
        dict::read_requestid_from(self.requestid_uref())
    }

    fn write_requestid(&self, requestid: U256) {
        dict::write_requestid_to(self.requestid_uref(), requestid)
    }

    fn get_dev_address(&self) -> Address {
        let _dev = self.read_dev();
        let _dev_str: &str = &_dev[..];
        Address::from(AccountHash::from_formatted_str(_dev_str).unwrap())
    }

    fn transfer_balance(
        &mut self,
        sender: Address,
        recipient: Address,
        amount: U256,
    ) -> Result<(), Error> {
        balances::transfer_balance(self.balances_uref(), sender, recipient, amount)
    }

    /// Installs the ERC20 contract with the default set of entry points.
    ///
    /// This should be called from within `fn call()` of your contract.
    pub fn install(
        name: String,
        symbol: String,
        decimals: u8,
        initial_supply: U256,
        minter: String,
        swap_fee: U256,
        dev: String,
        origin_chainid: U256,
        origin_contract_address: String,
    ) -> Result<ERC20, Error> {
        let default_entry_points = entry_points::default();
        ERC20::install_custom(
            name,
            symbol,
            decimals,
            initial_supply,
            minter,
            swap_fee,
            dev,
            origin_chainid,
            origin_contract_address,
            ERC20_TOKEN_CONTRACT_KEY_NAME,
            default_entry_points,
        )
    }

    /// Returns the name of the token.
    pub fn name(&self) -> String {
        detail::read_from(NAME_KEY_NAME)
    }

    /// Returns the origin chain of the wrapped token.
    pub fn origin_chainid(&self) -> U256 {
        detail::read_from("origin_chainid")
    }

    /// Returns the origin_contract_address of the wrapped token.
    pub fn origin_contract_address(&self) -> String {
        detail::read_from("origin_contract_address")
    }

    /// Returns the symbol of the token.
    pub fn symbol(&self) -> String {
        detail::read_from(SYMBOL_KEY_NAME)
    }

    /// Returns the minter of the token.
    pub fn minter(&self) -> String {
        detail::read_from(MINTER_KEY_NAME)
    }

    /// Returns the decimals of the token.
    pub fn decimals(&self) -> u8 {
        detail::read_from(DECIMALS_KEY_NAME)
    }

    /// Returns the total supply of the token.
    pub fn total_supply(&self) -> U256 {
        self.read_total_supply()
    }

    /// Returns the dev of the token.
    pub fn dev(&self) -> String {
        self.read_dev()
    }

    /// Returns the total supply of the token.
    pub fn swap_fee(&self) -> U256 {
        self.read_swap_fee()
    }

    /// Returns the balance of `owner`.
    pub fn balance_of(&self, owner: Address) -> U256 {
        self.read_balance(owner)
    }

    /// Transfers `amount` of tokens from the direct caller to `recipient`.
    pub fn transfer(&mut self, recipient: Address, amount: U256) -> Result<(), Error> {
        let sender = detail::get_immediate_caller_address()?;
        self.transfer_balance(sender, recipient, amount)
    }

    /// Transfers `amount` of tokens from `owner` to `recipient` if the direct caller has been
    /// previously approved to spend the specified amount on behalf of the owner.
    pub fn transfer_from(
        &mut self,
        owner: Address,
        recipient: Address,
        amount: U256,
    ) -> Result<(), Error> {
        let spender = detail::get_immediate_caller_address()?;
        if amount.is_zero() {
            return Ok(());
        }
        let spender_allowance = self.read_allowance(owner, spender);
        let new_spender_allowance = spender_allowance
            .checked_sub(amount)
            .ok_or(Error::InsufficientAllowance)?;
        self.transfer_balance(owner, recipient, amount)?;
        self.write_allowance(owner, spender, new_spender_allowance);
        Ok(())
    }

    /// Allows `spender` to transfer up to `amount` of the direct caller's tokens.
    pub fn approve(&mut self, spender: Address, amount: U256) -> Result<(), Error> {
        let owner = detail::get_immediate_caller_address()?;
        self.write_allowance(owner, spender, amount);
        Ok(())
    }

    /// Returns the amount of `owner`'s tokens allowed to be spent by `spender`.
    pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.read_allowance(owner, spender)
    }

    /// Mints `amount` new tokens and adds them to `owner`'s balance and to the token total supply.
    ///
    /// # Security
    ///
    /// This offers no security whatsoever, hence it is advised to NOT expose this method through a
    /// public entry point.
    /// unique_mint_key is concatenate of txhash(of request tx)|originchainid|tochainid|index
    pub fn mint(
        &mut self,
        owner: Address,
        amount: U256,
        swap_fee_in: U256,
        mintid: String,
    ) -> Result<(), Error> {
        let mintid_value = self.read_mintid(&mintid);
        if mintid_value > 0 {
            runtime::revert(Error::AlreadyMint);
        }
        self.write_mintid(&mintid);

        let _caller = detail::get_immediate_caller_address()?;
        let _caller_accounthash = _caller.as_account_hash().unwrap();
        let _minter = self.read_minter();
        if *_caller_accounthash.to_formatted_string() != _minter {
            runtime::revert(Error::NoAccessRights);
        }
        let swap_fee = self.read_swap_fee();
        if swap_fee != swap_fee_in {
            runtime::revert(Error::InvalidFee);
        }
        if amount < swap_fee {
            runtime::revert(Error::MintTooLow);
        }
        let mut new_balance = {
            let balance = self.read_balance(owner);
            balance.checked_add(amount).ok_or(Error::Overflow)?
        };
        new_balance = new_balance.checked_sub(swap_fee).ok_or(Error::Overflow)?;
        let new_total_supply = {
            let total_supply: U256 = self.read_total_supply();
            total_supply.checked_add(amount).ok_or(Error::Overflow)?
        };
        let _dev_addr = self.get_dev_address();
        let new_dev_balance = {
            let balance = self.read_balance(_dev_addr);
            balance.checked_add(swap_fee).ok_or(Error::Overflow)?
        };
        //mint fee
        self.write_balance(_dev_addr, new_dev_balance);
        self.write_balance(owner, new_balance);
        self.write_total_supply(new_total_supply);
        Ok(())
    }

    /// Change minter: only current minter can change
    pub fn change_minter(&mut self, new_minter: Address) -> Result<(), Error> {
        let _caller = detail::get_immediate_caller_address()?;
        let _caller_accounthash = _caller.as_account_hash().unwrap();
        let _current_minter = self.read_minter();
        if _caller_accounthash.to_formatted_string() != _current_minter {
            runtime::revert(Error::NoAccessRights);
        }
        let minter_accounthash = new_minter.as_account_hash().unwrap();
        self.write_minter(minter_accounthash.to_formatted_string());
        Ok(())
    }

    /// Change minter: only current minter can change
    pub fn change_dev(&mut self, new_dev: Address) -> Result<(), Error> {
        let _caller = detail::get_immediate_caller_address()?;
        let _caller_accounthash = _caller.as_account_hash().unwrap();
        let _current_dev = self.read_dev();
        if _caller_accounthash.to_formatted_string() != _current_dev {
            runtime::revert(Error::NoAccessRights);
        }
        let dev_accounthash = new_dev.as_account_hash().unwrap();
        self.write_dev(dev_accounthash.to_formatted_string());
        Ok(())
    }

    /// Change minter: only current minter can change
    pub fn change_swap_fee(&mut self, new_swap_fee: U256) -> Result<(), Error> {
        let _caller = detail::get_immediate_caller_address()?;
        let _caller_accounthash = _caller.as_account_hash().unwrap();
        let _current_dev = self.read_dev();
        if *_caller_accounthash.to_formatted_string() != _current_dev {
            runtime::revert(Error::NoAccessRights);
        }
        self.write_swap_fee(new_swap_fee);
        Ok(())
    }

    /// Burns (i.e. subtracts) `amount` of tokens from `owner`'s balance and from the token total
    /// supply.
    ///
    /// # Security
    ///
    /// This offers no security whatsoever, hence it is advised to NOT expose this method through a
    /// public entry point.
    pub fn burn(&mut self, amount: U256) -> Result<(), Error> {
        let _owner = detail::get_immediate_caller_address()?;
        self.burn_token(_owner, amount)
    }

    /// Burns (i.e. subtracts) `amount` to bridge back to original chain
    pub fn request_bridge_back(
        &mut self,
        amount: U256,
        fee: U256,
        _to_chain_id: U256,
        _receiver_address: String,
        unique_id: String,
    ) -> Result<(), Error> {
        //verify fee
        if fee != self.read_swap_fee() {
            runtime::revert(Error::InvalidFee);
        }

        if unique_id.chars().count() != 64 {
            runtime::revert(Error::RequestIdIllFormatted);
        }

        if hex::decode(&unique_id).is_err() {
            runtime::revert(Error::RequestIdIllFormatted);
        }

        //read request map
        let request_map_result = self.read_request_map(&unique_id);
        if request_map_result != U256::zero() {
            runtime::revert(Error::RequestIdExist);
        }
        //check whether id is used
        let val = self.read_requestid();
        let next_index = val + U256::one();

        self.write_requestid(next_index);
        self.write_request_map(&unique_id, next_index);

        let request_amount_after_fee =
            { amount.checked_sub(fee).ok_or(Error::RequestAmountTooLow)? };

        let _owner = detail::get_immediate_caller_address()?;

        //transfer fee to dev
        self.transfer_balance(_owner, self.get_dev_address(), fee)?;
        //burn the rest
        self.burn_token(_owner, request_amount_after_fee)?;
        Ok(())
    }

    /// Burns (i.e. subtracts) `amount` of tokens from `owner`'s balance and from the token total
    /// supply.
    ///
    /// # Security
    ///
    /// This offers no security whatsoever, hence it is advised to NOT expose this method through a
    /// public entry point.
    fn burn_token(&mut self, owner: Address, amount: U256) -> Result<(), Error> {
        let new_balance = {
            let balance = self.read_balance(owner);
            balance
                .checked_sub(amount)
                .ok_or(Error::InsufficientBalance)?
        };
        let new_total_supply = {
            let total_supply = self.read_total_supply();
            total_supply.checked_sub(amount).ok_or(Error::Overflow)?
        };
        self.write_balance(owner, new_balance);
        self.write_total_supply(new_total_supply);
        Ok(())
    }

    /// Installs the ERC20 contract with a custom set of entry points.
    ///
    /// # Warning
    ///
    /// Contract developers should use [`ERC20::install`] instead, as it will create the default set
    /// of ERC20 entry points. Using `install_custom` with a different set of entry points might
    /// lead to problems with integrators such as wallets, and exchanges.
    #[doc(hidden)]
    pub fn install_custom(
        name: String,
        symbol: String,
        decimals: u8,
        initial_supply: U256,
        minter: String,
        swap_fee: U256,
        dev: String,
        origin_chainid: U256,
        origin_contract_address: String,
        contract_key_name: &str,
        entry_points: EntryPoints,
    ) -> Result<ERC20, Error> {
        let balances_uref = storage::new_dictionary(BALANCES_KEY_NAME).unwrap_or_revert();
        let allowances_uref = storage::new_dictionary(ALLOWANCES_KEY_NAME).unwrap_or_revert();
        let mintids_uref = storage::new_dictionary("mintids").unwrap_or_revert();
        let requestid_uref = storage::new_uref(U256::zero()).into_read_write();
        // We need to hold on a RW access rights because tokens can be minted or burned.
        let total_supply_uref = storage::new_uref(initial_supply).into_read_write();
        let swap_fee_uref = storage::new_uref(swap_fee).into_read_write();
        let request_map_uref = storage::new_dictionary("request_map").unwrap_or_revert();

        let minter_uref = storage::new_uref(minter).into_read_write();
        let dev_uref = storage::new_uref(dev).into_read_write();

        let mut named_keys = NamedKeys::new();

        let name_key = {
            let name_uref = storage::new_uref(format!("{}", name)).into_read();
            Key::from(name_uref)
        };

        let symbol_key = {
            let symbol_uref = storage::new_uref(symbol).into_read();
            Key::from(symbol_uref)
        };

        let decimals_key = {
            let decimals_uref = storage::new_uref(decimals).into_read();
            Key::from(decimals_uref)
        };

        let origin_chainid_key = {
            let origin_chainid_uref = storage::new_uref(origin_chainid).into_read();
            Key::from(origin_chainid_uref)
        };

        let origin_contract_address_key = {
            let origin_contract_address_uref =
                storage::new_uref(origin_contract_address).into_read();
            Key::from(origin_contract_address_uref)
        };

        let total_supply_key = Key::from(total_supply_uref);
        let swap_fee_key = Key::from(swap_fee_uref);
        let minter_key = Key::from(minter_uref);
        let dev_key = Key::from(dev_uref);
        let requestid_key = Key::from(requestid_uref);

        let balances_dictionary_key = {
            // Sets up initial balance for the caller - either an account, or a contract.
            let caller = detail::get_caller_address()?;
            balances::write_balance_to(balances_uref, caller, initial_supply);

            runtime::remove_key(BALANCES_KEY_NAME);

            Key::from(balances_uref)
        };

        let allowances_dictionary_key = {
            runtime::remove_key(ALLOWANCES_KEY_NAME);

            Key::from(allowances_uref)
        };

        let mintids_dictionary_key = {
            runtime::remove_key("mintids");

            Key::from(mintids_uref)
        };

        let request_map_dictionary_key = {
            runtime::remove_key("request_map");

            Key::from(request_map_uref)
        };

        named_keys.insert(NAME_KEY_NAME.to_string(), name_key);
        named_keys.insert(SYMBOL_KEY_NAME.to_string(), symbol_key);
        named_keys.insert(DECIMALS_KEY_NAME.to_string(), decimals_key);
        named_keys.insert(BALANCES_KEY_NAME.to_string(), balances_dictionary_key);
        named_keys.insert(ALLOWANCES_KEY_NAME.to_string(), allowances_dictionary_key);
        named_keys.insert("mintids".to_string(), mintids_dictionary_key);
        named_keys.insert("requestid".to_string(), requestid_key);
        named_keys.insert("request_map".to_string(), request_map_dictionary_key);
        named_keys.insert(TOTAL_SUPPLY_KEY_NAME.to_string(), total_supply_key);
        named_keys.insert(MINTER_KEY_NAME.to_string(), minter_key);
        named_keys.insert("swap_fee".to_string(), swap_fee_key);
        named_keys.insert("dev".to_string(), dev_key);
        named_keys.insert("origin_chainid".to_string(), origin_chainid_key);
        named_keys.insert(
            "origin_contract_address".to_string(),
            origin_contract_address_key,
        );

        let (package_hash, access_token) = storage::create_contract_package_at_hash();
        
        let (contract_hash, _version) =
            storage::add_contract_version(package_hash, entry_points, named_keys);
        // let (contract_hash, _version) =
        //     storage::new_locked_contract(entry_points, Some(named_keys), None, None);

        // Store contract in the account's named keys.
        runtime::put_key(
          &format!("{}_contract_package_hash", name),
          package_hash.into(),
        );
        runtime::put_key(
          &format!("{}_contract_package_hash_wrapped", name),
          storage::new_uref(package_hash).into(),
        );
        runtime::put_key(
          &format!("{}_contract_contract_hash", name),
          contract_hash.into(),
        );
        runtime::put_key(
          &format!("{}_contract_contract_hash_wrapped", name),
          storage::new_uref(contract_hash).into(),
        );
        runtime::put_key(
          &format!("{}_contract_package_access_token", name),
          access_token.into(),
        );        

        // Hash of the installed contract will be reachable through named keys.
        runtime::put_key(contract_key_name, Key::from(contract_hash));

        Ok(ERC20::new(
            balances_uref,
            allowances_uref,
            total_supply_uref,
            minter_uref,
            swap_fee_uref,
            dev_uref,
            mintids_uref,
            requestid_uref,
            request_map_uref,
        ))
    }
}
