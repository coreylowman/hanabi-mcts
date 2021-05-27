extern crate rand;

mod env;
mod hanabi_env;
// mod mcts;
mod hanabi_distr;

use env::{Env, HasEnd, HasReward};
use hanabi_env::{Action, Card, CardCollection, HanabiEnv, Hint, PrivateInfo, PublicInfo};

use crate::rand::prelude::SliceRandom;
use crate::rand::rngs::StdRng;
use crate::rand::SeedableRng;

use std::time::Instant;

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
    let mut upper = std::f32::NEG_INFINITY;
    let mut lower = std::f32::INFINITY;
    let mut child_upper = Vec::new();
    let mut child_lower = Vec::new();
    let mut visits = Vec::new();

    for _ in 0..num_rollouts {
        let (action, reward) = rollout_fn(public_info.clone(), private_info.clone(), &mut rng);

        if upper < reward {
            upper = reward;
        }
        if lower > reward {
            lower = reward;
        }

        match actions.iter().position(|&a| a == action) {
            Some(i) => {
                rewards[i] += reward;
                visits[i] += 1;
                if child_upper[i] < reward {
                    child_upper[i] = reward;
                }
                if child_lower[i] > reward {
                    child_lower[i] = reward;
                }
            }
            None => {
                actions.push(action);
                rewards.push(reward);
                child_lower.push(reward);
                child_upper.push(reward);
                visits.push(1);
            }
        }
    }

    let mut best_i = 0;
    let mut best_score = std::f32::NEG_INFINITY;
    for i in 0..rewards.len() {
        let total_reward = rewards[i];
        // let mean_reward = total_reward / visits[i] as f32;
        // // let ugape = child_upper[i] - lower;
        // let mut B = std::f32::NEG_INFINITY;
        // for j in 0..rewards.len() {
        //     if i == j {
        //         continue;
        //     }
        //     let ugap = child_upper[j] - child_lower[i];
        //     if ugap > B {
        //         B = ugap;
        //     }
        // }
        // println!(
        //     "{:?}: {} / {} = {} | [{} {}]  | {}",
        //     actions[i], total_reward, visits[i], mean_reward, child_lower[i], child_upper[i], B,
        // );
        if total_reward > best_score {
            best_score = total_reward;
            best_i = i;
        }
        // if ugape > best_score {
        //     best_i = i;
        //     best_score = ugape;
        // }
    }

    actions[best_i]
}

fn describe_game<F: Fn(PublicInfo, PrivateInfo, &mut StdRng) -> (Action, f32)>(
    rollout_fn: &F,
    num_rollouts: usize,
) {
    let mut rng = StdRng::seed_from_u64(0);

    let mut env = HanabiEnv::random(&mut rng);

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
        // println!();
        // println!(">>> {:?}", action);
        // println!();
        env.step(&action, &mut rng);
        env.describe();
        println!();
    }
    println!("{} {}", env.reward(), env.fireworks.total());
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

        rewards.push(env.fireworks.total() as f32);

        let total_reward = rewards.iter().sum::<f32>();
        println!(
            "{} ({} / {})",
            total_reward / rewards.len() as f32,
            total_reward,
            rewards.len()
        );
    }
}

fn rollout_speed<F: Fn(PublicInfo, PrivateInfo, &mut StdRng) -> (Action, f32)>(
    rollout_fn: &F,
    num_rollouts: usize,
) {
    let mut rng = StdRng::seed_from_u64(0);
    let env = HanabiEnv::random(&mut rng);
    let public_info = env.public_info();
    let private_info = env.private_info(true);

    let mut times = Vec::new();
    loop {
        let start = Instant::now();

        for _ in 0..num_rollouts {
            rollout_fn(public_info.clone(), private_info.clone(), &mut rng);
        }

        let elapsed = start.elapsed().as_millis() as f32;
        times.push(elapsed);
        times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mean_time = times.iter().sum::<f32>() / times.len() as f32;
        let median_time = times[times.len() / 2];
        println!("mean={}ms | median={}ms", mean_time, median_time);
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

    // describe_game(&rollout_single_determinization, 500_000);
    evaluate(&rollout_single_determinization, 50_000);
    // rollout_speed(&rollout_single_determinization, 50_000);
}
