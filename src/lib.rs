#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Symbol};

#[contracttype]
#[derive(Clone)]
pub struct Bounty {
    pub creator: Address,
    pub amount: i128,
    pub open: bool,
}

#[contract]
pub struct SoroWaveEscrow;

#[contractimpl]
impl SoroWaveEscrow {
    pub fn initialize(env: Env, admin: Address, token_address: Address) {
        if env.storage().instance().has(&Symbol::new(&env, "admin")) {
            panic!("already initialized");
        }
        env.storage().instance().set(&Symbol::new(&env, "admin"), &admin);
        env.storage().instance().set(&Symbol::new(&env, "token"), &token_address);
    }

    pub fn create_bounty(env: Env, creator: Address, bounty_id: u64, amount: i128) {
        creator.require_auth();
        if amount <= 0 { panic!("amount must be positive"); }
        let key = (Symbol::new(&env, "bounty"), bounty_id);
        if env.storage().persistent().has(&key) { panic!("bounty already exists"); }
        let token_address: Address = env.storage().instance().get(&Symbol::new(&env, "token")).expect("not initialized");
        token::Client::new(&env, &token_address).transfer(&creator, &env.current_contract_address(), &amount);
        env.storage().persistent().set(&key, &Bounty {creator, amount, open:true});
    }

    pub fn release(env: Env, bounty_id: u64, contributor: Address) {
        let key = (Symbol::new(&env, "bounty"), bounty_id);
        let mut bounty: Bounty = env.storage().persistent().get(&key).expect("bounty not found");
        bounty.creator.require_auth();
        if !bounty.open { panic!("bounty is closed"); }
        bounty.open = false;
        env.storage().persistent().set(&key, &bounty);
        let token_address: Address = env.storage().instance().get(&Symbol::new(&env, "token")).expect("not initialized");
        token::Client::new(&env, &token_address).transfer(&env.current_contract_address(), &contributor, &bounty.amount);
    }

    pub fn cancel(env: Env, bounty_id: u64) {
        let key = (Symbol::new(&env, "bounty"), bounty_id);
        let mut bounty: Bounty = env.storage().persistent().get(&key).expect("bounty not found");
        bounty.creator.require_auth();
        if !bounty.open { panic!("bounty is closed"); }
        bounty.open = false;
        env.storage().persistent().set(&key, &bounty);
        let token_address: Address = env.storage().instance().get(&Symbol::new(&env, "token")).expect("not initialized");
        token::Client::new(&env, &token_address).transfer(&env.current_contract_address(), &bounty.creator, &bounty.amount);
    }
}
