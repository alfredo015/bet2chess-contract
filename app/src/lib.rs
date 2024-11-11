#![no_std]
// necesary crates
use sails_rs::{
    cell::RefCell, 
    gstd::{
        program,
        route,
        msg
    }, 
    prelude::*
};

// import our modules 
pub mod states;
pub mod services;

// import necesary data (CustomStruct state)
use states::{
    bet2chess_state::ChessState,
    signless_accounts_state::ContractSignlessAccounts
};


// Import service to be used for the program
use services::{
    bet2chess_service::Bet2ChessService,
    signless_service::SignlessService,
    query_service::QueryService
};


// Ping program struct to build the program (there can only be one per contract)
// Data is stored as a part of the program and passed to the services as Ref (query) 
// or RefMut (command), because services are instantiated for every incoming request
// message indicating that these services are stateless.
pub struct Bet2ChessProgram {
    bet2chess_state: RefCell<ChessState>,
    signless_state: RefCell<ContractSignlessAccounts>
}

// Ping program, it host one or more services and it expose them to the 
// externar consumer.
// Only one program is allowed per application
#[program]
impl Bet2ChessProgram {
    // Application constructor (it is an associated function)
    // It can be called once per application lifetime.
    pub fn new() -> Self {
        let mut chess_state = ChessState::default();
        chess_state.admins.push(msg::source());
        let bet2chess_state = RefCell::new(chess_state);
        let signless_state = RefCell::new(ContractSignlessAccounts::default());

        Self {
            bet2chess_state,
            signless_state
        }
    }

    #[route("Bet2Chess")]
    pub fn bet2chess_svc(&self) -> Bet2ChessService<'_> {
        Bet2ChessService::new(
            self.bet2chess_state.borrow_mut(), 
            self.signless_state.borrow()
        )
    }

    #[route("Signless")]
    pub fn signless_svc(&self) -> SignlessService<'_> {
        SignlessService::new(
            self.signless_state.borrow_mut()
        )
    }

    #[route("QueryService")]
    pub fn query_svc(&self) -> QueryService<'_> {
        QueryService::new(
            self.bet2chess_state.borrow(),
            // self.ping_state.borrow(),
            self.signless_state.borrow()
        )
    }
}