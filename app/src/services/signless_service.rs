use sails_rs::{
    gstd::{
        service,
        msg
    },
    cell::RefMut,
    prelude::*
};

// use crate::state_types::signless_accounts_state_type::{
//     ContractSignlessAccounts,
//     SignlessAccount,
//     SignlessError
// };

use crate::states::signless_accounts_state::{
    ContractSignlessAccounts,
    SignlessAccount,
    SignlessError
};

// #[derive(Default)]
pub struct SignlessService<'a> {
    data: RefMut<'a, ContractSignlessAccounts>
}

#[service]
impl<'a> SignlessService<'a> {
    pub fn new(data: RefMut<'a, ContractSignlessAccounts>) -> Self {
        Self {
            data
        }
    }

    pub fn bind_signless_data_to_address(
        &mut self, 
        user_address: ActorId,
        signless_data: SignlessAccount
    ) -> SignlessEvent {
        let signless_actor_id = msg::source().into();

        let result = self.data
            .set_signless_account_to_user_address(
                signless_actor_id, 
                user_address, 
                signless_data
            );

        match result {
            Err(signless_error) => SignlessEvent::Error(signless_error),
            Ok(_) => SignlessEvent::SignlessAccountSet
        }
    }

    pub fn bind_signless_data_to_no_wallet_account(
        &mut self,
        no_wallet_account: String,
        signless_data: SignlessAccount
    ) -> SignlessEvent {
        let signless_address: ActorId = msg::source().into();

        let result = self.data
            .set_signless_account_to_no_wallet_name(
                signless_address, 
                no_wallet_account, 
                signless_data
            );

        match result {
            Err(signless_error) => SignlessEvent::Error(signless_error),
            Ok(_) => SignlessEvent::SignlessAccountSet
        }
    }
}

#[derive(Encode, Decode, TypeInfo, Clone)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]

pub enum SignlessEvent {
    SignlessAccountSet,
    Error(SignlessError)
}