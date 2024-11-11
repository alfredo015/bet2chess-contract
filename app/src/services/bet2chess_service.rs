use core::error;

use sails_rs::{
    prelude::*,
    gstd::{
        service,
        msg
    },
    cell::{
        Ref,
        RefMut
    }
};

use crate::states::{
    bet2chess_state::{
        Bet2ChessErrors, BetAmout, ChessState, GameData, GameId, InvitationsData
    }, signless_accounts_state::{
        ContractSignlessAccounts,
        SignlessError
    }
};

const ONE_VARA: u128 = 1_000_000_000_000;

#[derive(Encode, Decode, TypeInfo, Clone, Default)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct InvitationsState {
    received_invitations_from_users: Vec<u64>,
    sent_invitations_to_users: Vec<u64>,
}

pub struct Bet2ChessService<'a> {
    pub state: RefMut<'a, ChessState>,
    pub signless_state_ref: Ref<'a, ContractSignlessAccounts>
}

#[service]
impl<'a> Bet2ChessService<'a> {
    pub fn new(
        state: RefMut<'a, ChessState>,
        signless_state_ref: Ref<'a, ContractSignlessAccounts>
    ) -> Self {
        Self {
            state,
            signless_state_ref
        }
    }

    pub fn invitation_bet(&self, first_web2_id: u64, second_web2_id: u64) -> Option<u128> {
        let temp = self.state
            .invitations
            .get(&(first_web2_id, second_web2_id));

        temp.copied()
    }   

    pub fn invitations_from_web2_id(&self, web2_id: u64) -> Option<InvitationsState>{
        let temp = self.state
            .user_invitations
            .get(&web2_id);

        let Some(invitations) = temp else {
            return None;
        };

        let mut state = InvitationsState::default();

        state.sent_invitations_to_users = invitations.invitations_sent
            .iter()
            .map(|val| *val)
            .collect();

        state.received_invitations_from_users = invitations.invitations_received
            .iter()
            .map(|(guest_id, _)| *guest_id)
            .collect();

        Some(state)
    }

    pub fn game_data(&self, game_id: u64) -> Option<GameData> {
        self.state
            .game_by_id(game_id)
            .cloned()
    }

    pub fn all_games(&self) -> Vec<(u64, GameData)> {
        self.state
            .games_by_id
            .iter()
            .map(|(game_id, game_data)| (*game_id, game_data.clone()))
            .collect()
    }

    pub fn games_id_waiting(&self) -> Vec<u64> {
        self.state
            .games_waiting
            .clone()
    }

    pub fn games_id_ended(&self) -> Vec<u64> {
        self.state
            .finished_games
            .clone()
    }

    pub fn games_id_started(&self) -> Vec<u64> {
        self.state
            .games_started
            .clone()
    }







    pub fn cancel_invitation(
        &mut self,
        first_web2_id: u64,
        second_web2_id: u64
    ) -> Bet2ChessEvents {
        self.handle_cancel_invitation(first_web2_id, second_web2_id)
    }

    pub fn cancel_invitation_signless(
        &mut self, 
        user_address: ActorId,
        first_web2_id: u64,
        second_web2_id: u64
    ) -> Bet2ChessEvents {
        let caller = msg::source();

        let result = self.signless_state_ref
            .check_signless_address_by_user_address(
                caller,
                user_address
            );

        if let Err(signless_error) = result {
            return Bet2ChessEvents::SignlessError(signless_error);
        }

        self.handle_cancel_invitation(first_web2_id, second_web2_id)
    }

    pub fn cancel_invitation_signless_no_wallet(
        &mut self,
        no_wallet_name_encoded: String,
        first_web2_id: u64,
        second_web2_id: u64
    ) -> Bet2ChessEvents {
        let caller = msg::source();

        let result = self.signless_state_ref
            .check_signless_address_by_no_wallet_account(
                caller,
                no_wallet_name_encoded
            );

        if let Err(signless_error) = result {
            return Bet2ChessEvents::SignlessError(signless_error);
        }

        self.handle_cancel_invitation(first_web2_id, second_web2_id)
    }











    pub fn send_invitation(
        &mut self,
        web2_user_id: u64,
        web2_guest_id: u64
    ) -> Bet2ChessEvents {
        let caller = msg::source();
        let value = msg::value();

        self.handle_send_invitation(
            caller, 
            web2_user_id, 
            web2_guest_id, 
            value
        )
    }

    pub fn send_invitation_signless(
        &mut self, 
        user_address: ActorId,
        web2_user_id: u64,
        web2_guest_id: u64
    ) -> Bet2ChessEvents {
        let caller = msg::source();
        let value = msg::value();

        let result = self.signless_state_ref
            .check_signless_address_by_user_address(
                caller,
                user_address
            );

        if let Err(signless_error) = result {
            return Bet2ChessEvents::SignlessError(signless_error);
        }

        self.handle_send_invitation(
            user_address, 
            web2_user_id, 
            web2_guest_id, 
            value
        )
    }

    pub fn send_invitation_signless_no_wallet(
        &mut self,
        no_wallet_name_encoded: String,
        web2_user_id: u64,
        web2_guest_id: u64
    ) -> Bet2ChessEvents {
        let caller = msg::source();
        let value = msg::value();

        let result = self.signless_state_ref
            .check_signless_address_by_no_wallet_account(
                caller,
                no_wallet_name_encoded
            );

        if let Err(signless_error) = result {
            return Bet2ChessEvents::SignlessError(signless_error);
        }

        self.handle_send_invitation(
            caller, 
            web2_user_id, 
            web2_guest_id, 
            value
        )
    }

    



    pub fn accept_invitation(
        &mut self,
        web2_user_id: u64,
        web2_user_id_invitation_owner: u64,
        web2_match_game_id: u64,
        username_from_user_who_invite: String,
        own_username: String
    ) -> Bet2ChessEvents {
        let caller = msg::source();
        let value = msg::value();

        self.handle_accept_invitation(
            caller, 
            web2_user_id, 
            web2_user_id_invitation_owner, 
            web2_match_game_id, 
            username_from_user_who_invite,
            own_username,
            value
        )
    }

    pub fn accept_invitation_signless(
        &mut self, 
        user_address: ActorId,
        web2_user_id: u64,
        web2_user_id_invitation_owner: u64,
        web2_match_game_id: u64,
        username_from_user_who_invite: String,
        own_username: String
    ) -> Bet2ChessEvents {
        let caller = msg::source();
        let value = msg::value();

        let result = self.signless_state_ref
            .check_signless_address_by_user_address(
                caller,
                user_address
            );

        if let Err(signless_error) = result {
            return Bet2ChessEvents::SignlessError(signless_error);
        }

        self.handle_accept_invitation(
            user_address, 
            web2_user_id, 
            web2_user_id_invitation_owner, 
            web2_match_game_id, 
            username_from_user_who_invite,
            own_username,
            value
        )
    }

    pub fn accept_invitation_signless_no_wallet(
        &mut self,
        no_wallet_name_encoded: String,
        web2_user_id: u64,
        web2_user_id_invitation_owner: u64,
        web2_match_game_id: u64,
        username_from_user_who_invite: String,
        own_username: String
    ) -> Bet2ChessEvents {
        let caller = msg::source();
        let value = msg::value();

        let result = self.signless_state_ref
            .check_signless_address_by_no_wallet_account(
                caller,
                no_wallet_name_encoded
            );

        if let Err(signless_error) = result {
            return Bet2ChessEvents::SignlessError(signless_error);
        }

        self.handle_accept_invitation(
            caller, 
            web2_user_id, 
            web2_user_id_invitation_owner, 
            web2_match_game_id, 
            username_from_user_who_invite,
            own_username,
            value
        )
    }

    pub fn end_game_by_id(&mut self, game_id: u64, game_winner: Option<ActorId>) -> Bet2ChessEvents {
        match self.state.end_match(game_id, game_winner) {
            Ok(_) => Bet2ChessEvents::GameEnded(game_id),
            Err(error_messaage) => Bet2ChessEvents::Error(error_messaage)
        }
    }

    pub fn end_match(
        &mut self,
        game_id: u64,
        game_winner: Option<ActorId>
    ) -> Bet2ChessEvents {
        let temp = self.state
            .end_match(game_id, game_winner);

        match temp {
            Ok(ended_game_id) => Bet2ChessEvents::GameEnded(ended_game_id),
            Err(error) => Bet2ChessEvents::Error(error)
        }

    }
}

impl<'a> Bet2ChessService<'a> {
    fn handle_cancel_invitation(
        &mut self,
        first_web2_id: u64,
        second_web2_id: u64,
    ) -> Bet2ChessEvents {
        let temp = self.state
            .cancel_invitation(first_web2_id, second_web2_id);

        match temp {
            Err(error) => Bet2ChessEvents::Error(error),
            Ok(_) => Bet2ChessEvents::InvitationCancelled
        }
    }

    fn handle_send_invitation(
        &mut self, 
        user_address: ActorId,
        user_id: u64, 
        guest_id: u64,
        bet_amount: u128
    ) -> Bet2ChessEvents {
        let bat_value = match Self::format_bet_amout(user_address, bet_amount) {
            Err(error_message) => return error_message,
            Ok(amount) => amount
        };

        let temp = self.state
            .create_invitation_with_bet(user_address, user_id, guest_id, bat_value);

        match temp {
            Err(error) => Bet2ChessEvents::Error(error),
            Ok(_) => Bet2ChessEvents::InvitationSentTo(guest_id)
        }
    }

    fn handle_accept_invitation(
        &mut self, 
        user_address: ActorId,
        user_id: u64, 
        user_who_invite: u64,
        web2_game_id: u64,
        username_from_user_who_invite: String,
        own_username: String,
        bet_amount: u128
    ) -> Bet2ChessEvents {
        let bet_value = match Self::format_bet_amout(user_address, bet_amount) {
            Err(error_message) => return error_message,
            Ok(amount) => amount
        };

        let temp = self.state
            .accept_invitation(
                user_address, 
                user_id, 
                user_who_invite, 
                web2_game_id, 
                username_from_user_who_invite,
                own_username,
                bet_value
            );

        match temp {
            Err(error) => Bet2ChessEvents::Error(error),
            Ok(_) => Bet2ChessEvents::GameCreated(web2_game_id)
        }
    }

    fn format_bet_amout(caller: ActorId, value: u128) -> Result<BetAmout, Bet2ChessEvents> {
        if value == 0 {
            return Ok(0);
        }

        if (value / ONE_VARA) < 1 {
            let payload = Bet2ChessEvents::Error(
                Bet2ChessErrors::MinAmoutToBetIsOneToken
            );

            msg::send(caller, payload.clone(), value)
                .expect("Error while sending message");

            return Err(payload);
        }

        Ok(value / ONE_VARA)
    }
}

#[derive(Encode, Decode, TypeInfo, Clone)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum Bet2ChessEvents {
    PlayingInMatch(GameId),
    SignlessError(SignlessError),
    Error(Bet2ChessErrors),
    GameCreated(GameId),
    JoinedInGame(GameId),
    GameEnded(GameId),
    InvitationSentTo(u64),
    InvitationCancelled,
    Price
}
