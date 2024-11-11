use sails_rs::{
    prelude::*,
    collections::BTreeMap
};

#[derive(Default)]
pub struct ContractSignlessAccounts {
    pub signless_accounts_address_by_user_address: BTreeMap<ActorId, ActorId>,
    pub signless_accounts_address_by_no_wallet_name: BTreeMap<String, ActorId>,
    pub signless_data_by_signless_address: BTreeMap<ActorId, SignlessAccount>
}

impl ContractSignlessAccounts {
    pub fn check_signless_address_by_user_address(
        &self,
        signless_address: ActorId,
        user_address: ActorId,
    ) -> Result<(), SignlessError> {
        let singless_addres_from_user_address = self
            .signless_accounts_address_by_user_address
            .get(&user_address)
            .ok_or(SignlessError::UserDoesNotHasSignlessAccount)?;

        if !signless_address.eq(singless_addres_from_user_address) {
            return Err(SignlessError::SessionHasInvalidSignlessAccount);
        }

        Ok(())
    }

    pub fn check_signless_address_by_no_wallet_account(
        &self,
        signless_address: ActorId,
        no_wallet_name_encoded: String
    ) -> Result<(), SignlessError> {
        let signless_address_by_no_wallet_account = self
            .signless_accounts_address_by_no_wallet_name
            .get(&no_wallet_name_encoded)
            .ok_or(SignlessError::UserDoesNotHasSignlessAccount)?;

        if !signless_address.eq(signless_address_by_no_wallet_account) {
            return Err(SignlessError::SessionHasInvalidSignlessAccount);
        }

        Ok(())
    }

    pub fn set_signless_account_to_user_address(
        &mut self, 
        signless_address: ActorId,
        user_address: ActorId,
        signless_data: SignlessAccount
    ) -> Result<(), SignlessError> {
        if self.signless_accounts_address_by_user_address.contains_key(&user_address) {
            return Err(SignlessError::UserAddressAlreadyExists);
        }

        if self.signless_data_by_signless_address.contains_key(&signless_address) {
            return Err(SignlessError::SignlessAddressAlreadyEsists);
        }

        self.add_signless_data_to_state(signless_address, signless_data);

        self
            .signless_accounts_address_by_user_address
            .insert(user_address, signless_address);

        Ok(())
    }

    pub fn set_signless_account_to_no_wallet_name(
        &mut self,
        signless_address: ActorId,
        no_wallet_name_encoded: String,
        signless_data: SignlessAccount
    ) -> Result<(), SignlessError> {
        if self.signless_accounts_address_by_no_wallet_name.contains_key(&no_wallet_name_encoded) {
            return Err(SignlessError::NoWalletAccountAlreadyExists);
        }

        if self.signless_data_by_signless_address.contains_key(&signless_address) {
            return Err(SignlessError::SignlessAddressAlreadyEsists);
        }

        self.add_signless_data_to_state(signless_address, signless_data);

        self
            .signless_accounts_address_by_no_wallet_name
            .insert(no_wallet_name_encoded, signless_address);

        Ok(())
    }

    pub fn add_signless_data_to_state(&mut self, signless_address: ActorId, signless_data: SignlessAccount) {
        self.signless_data_by_signless_address
            .insert(signless_address, signless_data);
    }
}

#[derive(Encode, Decode, TypeInfo, Clone)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum SignlessError {
    SignlessAccountHasInvalidSession,
    SignlessAccountNotApproved,
    SignlessAddressAlreadyEsists,
    UserAddressAlreadyExists,
    UserDoesNotHasSignlessAccount,
    NoWalletAccountAlreadyExists,
    NoWalletAccountDoesNotHasSignlessAccount,
    SessionHasInvalidSignlessAccount
}

#[derive(Encode, Decode, TypeInfo, Clone, Default)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct SignlessAccount {
    address: String,
    encoded: String,
}