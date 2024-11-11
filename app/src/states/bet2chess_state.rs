use sails_rs::{
    prelude::*,
    gstd::msg,
    collections::{
        BTreeMap,
        HashSet
    }
};

use crate::services::bet2chess_service::Bet2ChessEvents;

pub type UserWeb2Id = u64;
pub type GameId = u64;
pub type BetAmout = u128;

pub const ONE_VARA: u128 = 1_000_000_000_000;

// pub struct User {
//     address: Option<ActorId>,
//     user_web2_id: UserWeb2Id,
// }

#[derive(Encode, Decode, TypeInfo, Clone, Default)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct GameData {
    pub game_bet: u128,
    pub player1: ActorId,
    pub player2: ActorId,
    pub player1_username: String,
    pub player2_username: String,
    pub player1_web2_id: u64,
    pub player2_web2_id: u64,
    pub winner: Option<ActorId>,
    pub status: GameStatus,
}

pub struct InvitationsData {
    // Eso es por cuestiones de logica
    // El usuario que envie la solicitud unicamente sabra
    // el id web2 del usuario al que esta invitando
    // por eso unicamente se guarda la address
    // del usuario que invito, ya que cuando se acepte, de ahi
    // se obtendra la address
    pub invitations_sent: HashSet<UserWeb2Id>,
    pub invitations_received: BTreeMap<UserWeb2Id, ActorId>,
}

#[derive(Encode, Decode, TypeInfo, Clone, Default, Eq, PartialEq, Copy)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum GameStatus{
    #[default]
    Waiting,
    Started,
    Ended {
        winner: Option<ActorId>
    },
}

pub enum ResultEnd{
    Win,
    Lose,
    Draw,
}

#[derive(Default)]
pub struct ChessState {
    pub admins: Vec<ActorId>,
    pub games_started: Vec<GameId>,
    pub games_waiting: Vec<GameId>,
    pub finished_games: Vec<GameId>,
    pub games_by_web2_id: BTreeMap<u64, GameId>,
    pub games_by_id: BTreeMap<GameId, GameData>,
    pub invitations: BTreeMap<(UserWeb2Id, UserWeb2Id), BetAmout>,
    pub user_invitations: BTreeMap<UserWeb2Id, InvitationsData>,
    pub current_game_id: GameId
}

impl ChessState {
    pub fn game_by_id(&self, game_id: u64) -> Option<&GameData> {
        self.games_by_id
            .get(&game_id)
    }

    pub fn game_mut_by_id(&mut self, game_id: u64) -> Option<&mut GameData> {
        self.games_by_id.get_mut(&game_id)
    }

    pub fn cancel_invitation(&mut self, first_web2_id: UserWeb2Id, second_web2_id: UserWeb2Id) -> Result<(), Bet2ChessErrors> {
        self.user_invitations
            .get_mut(&first_web2_id)
            .ok_or(Bet2ChessErrors::UserHasNoInvitationfromTheUser(second_web2_id))?
            .invitations_sent
            .remove(&second_web2_id)
            .then(|| ())
            .ok_or_else(|| Bet2ChessErrors::UserHasNoInvitationfromTheUser(second_web2_id))?;

        let second_id_invitations = self.user_invitations
            .get_mut(&second_web2_id)
            .ok_or(Bet2ChessErrors::UserHasNoInvitationfromTheUser(first_web2_id))?;

        let first_user_address = second_id_invitations.invitations_received
            .remove(&first_web2_id)
            .ok_or(Bet2ChessErrors::UserHasNoInvitationfromTheUser(first_web2_id))?;

        let bet_amount = self.invitations
            .remove(&(first_web2_id, second_web2_id))
            .ok_or(Bet2ChessErrors::InvitationDoesNotExists)?;
    
        if bet_amount != 0 {
            msg::send(first_user_address, b"Refund", bet_amount * ONE_VARA)
                .expect("Error while sending a messge");
        }

        Ok(())
    }

    pub fn create_invitation_with_bet(&mut self, user_address: ActorId, user_id: UserWeb2Id, guest: UserWeb2Id, bet_amount: u128) -> Result<(), Bet2ChessErrors> {
        // Se obtiene las invitaciones del usuario, si no existe, se 
        // inserta como nuevo usuario y se retorna sus invitaciones como mutables.
        let invitations = self.user_invitations
            .entry(user_id)
            .or_insert(InvitationsData {
                invitations_sent: HashSet::new(),
                invitations_received: BTreeMap::new()
            });

        // Se verifica si el usuario ya a mandado 
        // la invitacion al usuario 
        let already_sent_invitation = invitations.invitations_sent
            .contains(&guest);

        // Si es el caso, se retorna error y se retornan los tokens
        if already_sent_invitation {
            if bet_amount != 0 {
                msg::send(
                    user_address, 
                    Bet2ChessErrors::UserAlreadyInviteThePlayer(guest), 
                    bet_amount * ONE_VARA
                ).expect("Error while sending a message");
            }

            return Err(Bet2ChessErrors::UserAlreadyInviteThePlayer(guest));
        }

        // Se agrega la nueva invitacion que se mando
        // al usuario que mando a llamar al contrato
        invitations.invitations_sent    
            .insert(guest);
        
        // Se obtienen las invitaciones del usuario al que se le mando la invitacion,
        // en caso de no existir, se crea y se retorna sus invitaciones como mutables
        let guest_invitations = self.user_invitations
            .entry(guest)
            .or_insert(InvitationsData {
                invitations_sent: HashSet::new(),
                invitations_received: BTreeMap::new()
            });
        
        // Se verifica si el usuario ya a sido invitado
        // paso para seguridad extra
        let already_invited = guest_invitations.invitations_received
            .contains_key(&user_id);
        
        // Si es el caso, se retorna error
        // y se borra la invitacion guardada del jugador que mando
        // la apuesta
        if already_invited {
            // Se hace asi por el borrowing
            self.user_invitations
                .entry(user_id)
                .and_modify(|invitations| {
                    invitations.invitations_sent
                        .remove(&guest);
                });

            if bet_amount != 0 {
                msg::send(
                    user_address, 
                    Bet2ChessErrors::UserAlreadyInviteThePlayer(guest), 
                    bet_amount * ONE_VARA
                ).expect("Error while sending a message");
            }
            
            return Err(Bet2ChessErrors::UserAlreadyInviteThePlayer(guest));
        }

        // Se agrega la nueva invitacion que el usuario recibio
        guest_invitations.invitations_received
            .insert(user_id, user_address);

        // Se crea la invitacion "global", para saber el monton de la
        // apuesta. 
        self.invitations.insert((user_id, guest), bet_amount);

        Ok(())
    }

    pub fn accept_invitation(
        &mut self, 
        user_address: ActorId,
        invited_user: UserWeb2Id, 
        user_who_invite: UserWeb2Id, 
        web2_game_id: u64, 
        username_from_user_who_invite: String,
        own_username: String,
        bet_amount: BetAmout
    ) -> Result<(), Bet2ChessErrors> {
        // Si no encuentra la invitacion, manda error ya que no existe una invitacion como tal,
        // si existe, se retorna la apueta propuesta por el jugador.
        let bet = self.invitations
            .get_mut(&(user_who_invite, invited_user))
            .ok_or(Bet2ChessErrors::UserHasNoInvitationfromTheUser(user_who_invite))?;
        
        // Se verifica que el jugador que acepto la partida haya mandado la cantidad
        // propuesta
        if *bet != bet_amount {
            return Err(Bet2ChessErrors::BetIsNotTheSameForMatch { 
                game_bet: *bet, 
                bet_by_user: bet_amount 
            });
        }

        // Se elimina la invitaion del contrato
        self.invitations
            .remove(&(user_who_invite, invited_user));

        // Se elimina la invitacion por parte del usuario que mando 
        // la invitacion.
        self.user_invitations
            .entry(user_who_invite)
            .and_modify(|invitations| {
                invitations.invitations_sent.remove(&invited_user);
            });
        

        // Se elimina la invitacion y se obtiene la address del usuario
        // que invito al jugador.
        let temp = self.user_invitations
            .get_mut(&invited_user)
            .unwrap()
            .invitations_received
            .remove(&user_who_invite);

        // Si este no existia, se retorna error, y en caso de
        // apuesta, se retorna
        let Some(first_user_address) = temp else {
            if bet_amount != 0 {
                msg::send(
                    user_address, 
                    Bet2ChessErrors::UserHasNoInvitationfromTheUser(user_who_invite), 
                    bet_amount * ONE_VARA
                ).expect("Error while sending a message");
            }

            return Err(Bet2ChessErrors::UserHasNoInvitationfromTheUser(user_who_invite));
        };

        // Se crea la partida y se une a ambos jugadores a esta.
        let _ = Self::create_match(
            self, 
            first_user_address, 
            username_from_user_who_invite,
            user_who_invite,
            bet_amount, 
            web2_game_id
        )?;
        let _ = Self::join_match(
            self, 
            user_address, 
            own_username,
            invited_user,
            bet_amount, 
            web2_game_id
        )?;

        Ok(())  
    }

    pub fn join_match(
        &mut self, 
        address: ActorId, 
        username: String,
        user_web2_id: u64,
        bet_amount: BetAmout, 
        game_id: u64
    ) -> Result<(), Bet2ChessErrors> {
        let game_data = self.game_mut_by_id(game_id)
            .ok_or(Bet2ChessErrors::GameIdDoesNotExists(game_id))?;

        if game_data.status == GameStatus::Started {
            return Err(Bet2ChessErrors::GameAlreadyStart(game_id));
        }

        if game_data.game_bet != bet_amount {
            return Err(Bet2ChessErrors::BetIsNotTheSameForMatch{
                game_bet: game_data.game_bet,
                bet_by_user: bet_amount
            });
        }

        game_data.player2 = address;
        game_data.player2_username = username;
        game_data.player2_web2_id = user_web2_id;
        game_data.status = GameStatus::Started;

        self.games_started.push(game_id);

        Ok(())
    }

    pub fn create_match(
        &mut self, 
        address: ActorId,
        username: String,
        user_web2_id: u64,
        bet_amount: BetAmout,
        game_id: u64
    ) -> Result<GameId, Bet2ChessErrors> {
        let mut game_data = Self::new_game_with_bet(bet_amount);
        game_data.player1 = address;
        game_data.player1_username = username;
        game_data.player1_web2_id = user_web2_id;

        self.games_by_id.insert(game_id, game_data);
        self.games_waiting.push(game_id);
        // self.current_game_id = new_current_game_id;

        Ok(game_id)
    }

    pub fn end_match(
        &mut self,
        game_id: GameId,
        // caller: ActorId,
        game_winner: Option<ActorId>
    ) -> Result<GameId, Bet2ChessErrors> {
        // Esta comentado por como esta el flujo en el frontend
        // self.admins
        //     .iter()
        //     .find(|&&admin| admin == caller)
        //     .ok_or(Bet2ChessErrors::OnlyAdminsCanEndGames)?;

        let game_data = self.games_by_id
            .get_mut(&game_id)
            .ok_or(Bet2ChessErrors::GameIdDoesNotExists(game_id))?;

        let Some(winner) = game_winner else {
            game_data.status = GameStatus::Ended { 
                winner: None 
            };

            return Ok(game_id);
        };

        if game_data.player1 == winner {
            game_data.status = GameStatus::Ended { 
                winner:Some(game_data.player1) 
            };    
        } else {
            game_data.status = GameStatus::Ended { 
                winner:Some(game_data.player2) 
            };
        }       

        msg::send(winner, Bet2ChessEvents::Price, game_data.game_bet * ONE_VARA * 2)
            .expect("Error while sending message");

        Ok(game_id)
    }

    fn new_game_with_bet(bet_amount: BetAmout) -> GameData {
        let mut game_data = GameData::default();
        game_data.game_bet = bet_amount;

        game_data
    }
}


#[derive(Encode, Decode, TypeInfo, Clone)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum Bet2ChessErrors {
    GameIdDoesNotExists(GameId),
    GameWithIdAlreadyStarts(GameId),
    GameAlreadyStart(GameId),
    BetIsNotTheSameForMatch {
        game_bet: u128,
        bet_by_user: u128
    },
    UserAlreadyInviteThePlayer(UserWeb2Id),
    UserHasNoInvitationfromTheUser(UserWeb2Id),
    UserAddressAndWeb2IdAreNotRelated,
    CantIncrementGamesIdItOverflow,
    InvitationDoesNotExists,
    ThereAreNoGamesWaiting,
    MinAmoutToBetIsOneToken,
    OnlyAdminsCanEndGames
}