#![no_std]

use soroban_sdk::{contract, contractevent, contractimpl, contracttype, token, Address, Env};

const TTL_THRESHOLD: u32 = 100_000;
const TTL_EXTEND_TO: u32 = 5_000_000;

fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
}

fn extend_persistent_ttl(env: &Env, key: &DataKey) {
    env.storage()
        .persistent()
        .extend_ttl(key, TTL_THRESHOLD, TTL_EXTEND_TO);
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WavePool {
    pub creator: Address,
    pub initial_pool: i128,
    pub remaining_pool: i128,
    pub total_points: u64,
    pub remaining_points: u64,
    pub finalized: bool,
    pub cancelled: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationAward {
    pub contributor: Address,
    pub points: u64,
}

#[contractevent]
pub struct Initialized {
    #[topic]
    admin: Address,
    token_address: Address,
}

#[contractevent]
pub struct WaveCreated {
    #[topic]
    wave_id: u64,
    amount: i128,
}

#[contractevent]
pub struct PointsAwarded {
    #[topic]
    wave_id: u64,
    #[topic]
    contributor: Address,
    points: u64,
}

#[contractevent]
pub struct ApplicationPointsAwarded {
    #[topic]
    wave_id: u64,
    #[topic]
    application_id: soroban_sdk::String,
    #[topic]
    contributor: Address,
    points: u64,
}

#[contractevent]
pub struct WaveFinalized {
    #[topic]
    wave_id: u64,
    total_points: u64,
}

#[contractevent]
pub struct RewardClaimed {
    #[topic]
    wave_id: u64,
    #[topic]
    contributor: Address,
    payout: i128,
}

#[contractevent]
pub struct WaveCancelled {
    #[topic]
    wave_id: u64,
    amount: i128,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Token,
    Wave(u64),
    Points(u64, Address),
    ApplicationAward(u64, soroban_sdk::String),
    Claimed(u64, Address),
}

#[contract]
pub struct SoroWavePool;

#[contractimpl]
impl SoroWavePool {
    pub fn initialize(env: Env, admin: Address, token_address: Address) {
        let key = DataKey::Admin;
        if env.storage().instance().has(&key) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&key, &admin);
        env.storage()
            .instance()
            .set(&DataKey::Token, &token_address);
        extend_instance_ttl(&env);
        Initialized {
            admin,
            token_address,
        }
        .publish(&env);
    }

    /// Escrows the reward pool for one contribution cycle.
    pub fn create_wave(env: Env, creator: Address, wave_id: u64, amount: i128) {
        creator.require_auth();
        extend_instance_ttl(&env);
        if amount <= 0 {
            panic!("pool amount must be positive");
        }
        let key = DataKey::Wave(wave_id);
        if env.storage().persistent().has(&key) {
            panic!("wave already exists");
        }
        let token_address: Address = env
            .storage()
            .instance()
            .get(&DataKey::Token)
            .expect("not initialized");
        token::Client::new(&env, &token_address).transfer(
            &creator,
            &env.current_contract_address(),
            &amount,
        );
        let pool = WavePool {
            creator: creator.clone(),
            initial_pool: amount,
            remaining_pool: amount,
            total_points: 0,
            remaining_points: 0,
            finalized: false,
            cancelled: false,
        };
        env.storage().persistent().set(&key, &pool);
        extend_persistent_ttl(&env, &key);
        WaveCreated { wave_id, amount }.publish(&env);
    }

    /// Records points after a maintainer verifies completed work.
    pub fn award_points(env: Env, wave_id: u64, contributor: Address, points: u64) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");
        extend_instance_ttl(&env);
        admin.require_auth();
        if points == 0 {
            panic!("points must be positive");
        }
        let wave_key = DataKey::Wave(wave_id);
        let mut wave: WavePool = env
            .storage()
            .persistent()
            .get(&wave_key)
            .expect("wave not found");
        extend_persistent_ttl(&env, &wave_key);
        if wave.finalized || wave.cancelled {
            panic!("wave is closed");
        }
        let points_key = DataKey::Points(wave_id, contributor.clone());
        let current: u64 = env.storage().persistent().get(&points_key).unwrap_or(0);
        let updated_contributor_points = current
            .checked_add(points)
            .expect("contributor points overflow");
        wave.total_points = wave
            .total_points
            .checked_add(points)
            .expect("wave points overflow");
        wave.remaining_points = wave
            .remaining_points
            .checked_add(points)
            .expect("remaining points overflow");
        env.storage()
            .persistent()
            .set(&points_key, &updated_contributor_points);
        extend_persistent_ttl(&env, &points_key);
        env.storage().persistent().set(&wave_key, &wave);
        extend_persistent_ttl(&env, &wave_key);
        PointsAwarded {
            wave_id,
            contributor,
            points,
        }
        .publish(&env);
    }

    /// Records a reviewed application's points once. Repeating an identical
    /// award is safe; reusing its id for different points or a different
    /// contributor is rejected.
    pub fn award_application_points(
        env: Env,
        wave_id: u64,
        application_id: soroban_sdk::String,
        contributor: Address,
        points: u64,
    ) -> bool {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");
        extend_instance_ttl(&env);
        admin.require_auth();
        if application_id.is_empty() || points == 0 {
            panic!("application id and positive points are required");
        }
        let award_key = DataKey::ApplicationAward(wave_id, application_id.clone());
        if let Some(previous) = env
            .storage()
            .persistent()
            .get::<_, ApplicationAward>(&award_key)
        {
            if previous.contributor == contributor && previous.points == points {
                extend_persistent_ttl(&env, &award_key);
                // Re-emit proof so the API can validate a safe retry after the
                // award succeeded but the previous API update failed.
                ApplicationPointsAwarded {
                    wave_id,
                    application_id,
                    contributor,
                    points,
                }
                .publish(&env);
                return false;
            }
            panic!("application award conflicts with its prior award");
        }
        let wave_key = DataKey::Wave(wave_id);
        let mut wave: WavePool = env
            .storage()
            .persistent()
            .get(&wave_key)
            .expect("wave not found");
        extend_persistent_ttl(&env, &wave_key);
        if wave.finalized || wave.cancelled {
            panic!("wave is closed");
        }
        let points_key = DataKey::Points(wave_id, contributor.clone());
        let current: u64 = env.storage().persistent().get(&points_key).unwrap_or(0);
        let updated_contributor_points = current
            .checked_add(points)
            .expect("contributor points overflow");
        wave.total_points = wave
            .total_points
            .checked_add(points)
            .expect("wave points overflow");
        wave.remaining_points = wave
            .remaining_points
            .checked_add(points)
            .expect("remaining points overflow");
        env.storage()
            .persistent()
            .set(&points_key, &updated_contributor_points);
        extend_persistent_ttl(&env, &points_key);
        env.storage().persistent().set(&wave_key, &wave);
        extend_persistent_ttl(&env, &wave_key);
        env.storage().persistent().set(
            &award_key,
            &ApplicationAward {
                contributor: contributor.clone(),
                points,
            },
        );
        extend_persistent_ttl(&env, &award_key);
        ApplicationPointsAwarded {
            wave_id,
            application_id,
            contributor,
            points,
        }
        .publish(&env);
        true
    }

    /// Locks the points total so contributors can claim their pro-rata share.
    pub fn finalize_wave(env: Env, wave_id: u64) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");
        extend_instance_ttl(&env);
        admin.require_auth();
        let key = DataKey::Wave(wave_id);
        let mut wave: WavePool = env
            .storage()
            .persistent()
            .get(&key)
            .expect("wave not found");
        extend_persistent_ttl(&env, &key);
        if wave.cancelled || wave.finalized {
            panic!("wave is closed");
        }
        if wave.total_points == 0 {
            panic!("wave has no awarded points");
        }
        wave.finalized = true;
        env.storage().persistent().set(&key, &wave);
        extend_persistent_ttl(&env, &key);
        WaveFinalized {
            wave_id,
            total_points: wave.total_points,
        }
        .publish(&env);
    }

    /// Pays a contributor a pro-rata share of the remaining pool.
    /// Integer-division remainder is carried forward to later claimants.
    pub fn claim(env: Env, wave_id: u64, contributor: Address) -> i128 {
        contributor.require_auth();
        extend_instance_ttl(&env);
        let wave_key = DataKey::Wave(wave_id);
        let mut wave: WavePool = env
            .storage()
            .persistent()
            .get(&wave_key)
            .expect("wave not found");
        extend_persistent_ttl(&env, &wave_key);
        if !wave.finalized || wave.cancelled {
            panic!("wave is not claimable");
        }
        let claimed_key = DataKey::Claimed(wave_id, contributor.clone());
        if env.storage().persistent().has(&claimed_key) {
            panic!("already claimed");
        }
        let points_key = DataKey::Points(wave_id, contributor.clone());
        let points: u64 = env.storage().persistent().get(&points_key).unwrap_or(0);
        if points > 0 {
            extend_persistent_ttl(&env, &points_key);
        }
        if points == 0 || wave.remaining_points == 0 {
            panic!("no points to claim");
        }
        let payout = if points == wave.remaining_points {
            wave.remaining_pool
        } else {
            wave.remaining_pool
                .checked_mul(points as i128)
                .expect("payout overflow")
                / wave.remaining_points as i128
        };
        wave.remaining_pool -= payout;
        wave.remaining_points -= points;
        env.storage().persistent().set(&claimed_key, &true);
        extend_persistent_ttl(&env, &claimed_key);
        env.storage().persistent().set(&wave_key, &wave);
        extend_persistent_ttl(&env, &wave_key);
        if payout > 0 {
            let token_address: Address = env
                .storage()
                .instance()
                .get(&DataKey::Token)
                .expect("not initialized");
            token::Client::new(&env, &token_address).transfer(
                &env.current_contract_address(),
                &contributor,
                &payout,
            );
        }
        RewardClaimed {
            wave_id,
            contributor,
            payout,
        }
        .publish(&env);
        payout
    }

    /// Returns the unused pool only when no points have been awarded.
    pub fn cancel_wave(env: Env, wave_id: u64) {
        let key = DataKey::Wave(wave_id);
        let mut wave: WavePool = env
            .storage()
            .persistent()
            .get(&key)
            .expect("wave not found");
        extend_persistent_ttl(&env, &key);
        wave.creator.require_auth();
        if wave.finalized || wave.cancelled || wave.total_points > 0 {
            panic!("wave cannot be cancelled");
        }
        wave.cancelled = true;
        let amount = wave.remaining_pool;
        wave.remaining_pool = 0;
        env.storage().persistent().set(&key, &wave);
        extend_persistent_ttl(&env, &key);
        extend_instance_ttl(&env);
        let token_address: Address = env
            .storage()
            .instance()
            .get(&DataKey::Token)
            .expect("not initialized");
        token::Client::new(&env, &token_address).transfer(
            &env.current_contract_address(),
            &wave.creator,
            &amount,
        );
        WaveCancelled { wave_id, amount }.publish(&env);
    }

    pub fn get_wave(env: Env, wave_id: u64) -> Option<WavePool> {
        let key = DataKey::Wave(wave_id);
        let wave = env.storage().persistent().get(&key);
        if wave.is_some() {
            extend_persistent_ttl(&env, &key);
        }
        wave
    }

    pub fn get_points(env: Env, wave_id: u64, contributor: Address) -> u64 {
        let key = DataKey::Points(wave_id, contributor);
        let points = env.storage().persistent().get(&key).unwrap_or(0);
        if points > 0 {
            extend_persistent_ttl(&env, &key);
        }
        points
    }

    pub fn has_claimed(env: Env, wave_id: u64, contributor: Address) -> bool {
        let key = DataKey::Claimed(wave_id, contributor);
        let claimed = env.storage().persistent().has(&key);
        if claimed {
            extend_persistent_ttl(&env, &key);
        }
        claimed
    }
}
