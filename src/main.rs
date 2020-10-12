extern crate rand;

mod env;
mod hanabi_env;
// mod mcts;

use env::{Env, HasEnd, HasReward};
use hanabi_env::{Action, Card, CardCollection, HanabiEnv, Hint, PrivateInfo, PublicInfo};

use crate::rand::prelude::SliceRandom;
use crate::rand::rngs::StdRng;
use crate::rand::SeedableRng;

fn rollout(
    mut public_info: PublicInfo,
    mut my_private: PrivateInfo,
    mut rng: &mut StdRng,
) -> (Action, f32) {
    let mut env = HanabiEnv::determinize(&public_info, &my_private, &mut rng);
    let action = *env.actions().choose(&mut rng).unwrap();
    env.step(&action, &mut rng);
    public_info = env.public_info();
    assert_eq!(env.private_info(false), my_private);
    let mut op_private = env.private_info(true);

    let mut my_turn = false;

    loop {
        if public_info.is_over() {
            break;
        }

        if my_turn {
            env = HanabiEnv::determinize(&public_info, &my_private, &mut rng);
            env.step(env.actions().choose(&mut rng).unwrap(), &mut rng);
            public_info = env.public_info();
            assert_eq!(env.private_info(false), my_private);
            op_private = env.private_info(true);
        } else {
            env = HanabiEnv::determinize(&public_info, &op_private, &mut rng);
            env.step(env.actions().choose(&mut rng).unwrap(), &mut rng);
            public_info = env.public_info();
            assert_eq!(env.private_info(false), op_private);
            my_private = env.private_info(true);
        }

        my_turn = !my_turn;
    }

    (action, public_info.reward())
}

fn policy(public_info: PublicInfo, private_info: PrivateInfo, mut rng: &mut StdRng) -> Action {
    let mut actions = Vec::new();
    let mut rewards = Vec::new();
    let mut visits = Vec::new();

    for _ in 0..200_000 {
        let (action, reward) = rollout(public_info.clone(), private_info.clone(), &mut rng);

        match actions.iter().position(|&a| a == action) {
            Some(i) => {
                rewards[i] += reward;
                visits[i] += 1;
            }
            None => {
                actions.push(action);
                rewards.push(reward);
                visits.push(1);
            }
        }
    }

    let mut best_i = 0;
    let mut best_score = std::f32::NEG_INFINITY;
    for i in 0..rewards.len() {
        let total_reward = rewards[i];
        let mean_reward = total_reward / visits[i] as f32;
        println!(
            "{:?}: {} ({} / {})",
            actions[i], mean_reward, total_reward, visits[i],
        );
        if mean_reward > best_score {
            best_i = i;
            best_score = mean_reward;
        }
    }

    actions[best_i]
}

fn main() {
    println!("Card {}", std::mem::size_of::<Card>());
    println!("Hint {}", std::mem::size_of::<Hint>());
    println!("CardCollection {}", std::mem::size_of::<CardCollection>());
    println!("Env {}", std::mem::size_of::<HanabiEnv>());
    println!("PublicInfo {}", std::mem::size_of::<PublicInfo>());
    println!("PrivateInfo {}", std::mem::size_of::<PrivateInfo>());
    println!();

    let mut rng = StdRng::seed_from_u64(0);

    loop {
        let mut env = HanabiEnv::new(&mut rng);
        env.describe();
        println!();

        while !env.is_over() {
            let action = policy(env.public_info(), env.private_info(true), &mut rng);
            println!();
            env.describe();
            println!("{:?}", action);
            env.step(&action, &mut rng);
            env.describe();
            println!();
        }
        println!("{} {}", env.reward(), env.fireworks.iter().sum::<u8>());
    }
}
