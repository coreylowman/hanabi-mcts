extern crate rand;

mod env;
mod hanabi_env;
// mod mcts;

use env::{Env, HasEnd, HasReward};
use hanabi_env::{Action, Card, CardCollection, HanabiEnv, Hint, PrivateInfo, PublicInfo};

use crate::rand::prelude::SliceRandom;
use crate::rand::rngs::StdRng;
use crate::rand::SeedableRng;

fn rollout_single_determinization(
    public_info: PublicInfo,
    my_private: PrivateInfo,
    mut rng: &mut StdRng,
) -> (Action, f32) {
    let (mut env, prob) = HanabiEnv::determinize(&public_info, &my_private, &mut rng);
    let action = *env.actions().choose(&mut rng).unwrap();
    env.step(&action, &mut rng);

    while !env.is_over() {
        env.step(env.actions().choose(&mut rng).unwrap(), &mut rng);
    }

    (action, prob * env.reward())
}

fn policy<F: Fn(PublicInfo, PrivateInfo, &mut StdRng) -> (Action, f32)>(
    public_info: PublicInfo,
    private_info: PrivateInfo,
    rollout_fn: &F,
    num_rollouts: usize,
    mut rng: &mut StdRng,
) -> Action {
    let mut actions = Vec::new();
    let mut rewards = Vec::new();
    let mut visits = Vec::new();

    for _ in 0..num_rollouts {
        let (action, reward) = rollout_fn(public_info.clone(), private_info.clone(), &mut rng);

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
        // println!(
        //     "{:?}: {} ({} / {})",
        //     actions[i], mean_reward, total_reward, visits[i],
        // );
        if mean_reward > best_score {
            best_i = i;
            best_score = mean_reward;
        }
    }

    actions[best_i]
}

fn describe_game<F: Fn(PublicInfo, PrivateInfo, &mut StdRng) -> (Action, f32)>(
    rollout_fn: &F,
    num_rollouts: usize,
) {
    let mut rng = StdRng::seed_from_u64(0);

    let mut env = HanabiEnv::random(&mut rng);
    env.describe();
    println!();

    while !env.is_over() {
        let action = policy(
            env.public_info(),
            env.private_info(true),
            rollout_fn,
            num_rollouts,
            &mut rng,
        );
        println!();
        env.describe();
        println!("{:?}", action);
        env.step(&action, &mut rng);
        env.describe();
        println!();
    }
    println!("{} {}", env.reward(), env.fireworks.iter().sum::<u8>());
}

fn evaluate<F: Fn(PublicInfo, PrivateInfo, &mut StdRng) -> (Action, f32)>(
    rollout_fn: &F,
    num_rollouts: usize,
) {
    let mut rng = StdRng::seed_from_u64(0);

    let mut rewards = Vec::new();

    for _ in 0..100 {
        let mut env = HanabiEnv::random(&mut rng);

        while !env.is_over() {
            let action = policy(
                env.public_info(),
                env.private_info(true),
                rollout_fn,
                num_rollouts,
                &mut rng,
            );
            env.step(&action, &mut rng);
        }

        rewards.push(env.fireworks.iter().sum::<u8>() as f32);

        let total_reward = rewards.iter().sum::<f32>();
        println!(
            "{} ({} / {})",
            total_reward / rewards.len() as f32,
            total_reward,
            rewards.len()
        );
    }
}

fn main() {
    println!("Card {}", std::mem::size_of::<Card>());
    println!("Hint {}", std::mem::size_of::<Hint>());
    println!("CardCollection {}", std::mem::size_of::<CardCollection>());
    println!("Env {}", std::mem::size_of::<HanabiEnv>());
    println!("PublicInfo {}", std::mem::size_of::<PublicInfo>());
    println!("PrivateInfo {}", std::mem::size_of::<PrivateInfo>());
    println!();

    // describe_game(&rollout_single_determinization, 50_000);
    evaluate(&rollout_single_determinization, 50_000);
}
