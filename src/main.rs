extern crate rand;

mod env;
mod hanabi_env;
// mod mcts;

use env::Env;
use hanabi_env::{Action, Card, CardCollection, HanabiEnv, Hints, PublicInfo};

use crate::rand::prelude::SliceRandom;
use crate::rand::rngs::StdRng;
use crate::rand::SeedableRng;

fn policy(root_public_info: &PublicInfo, mut rng: &mut StdRng) -> Action {
    let mut actions = Vec::new();
    let mut rewards = Vec::new();
    let mut visits = Vec::new();

    for _ in 0..100_000 {
        let (action, reward) = rollout(root_public_info, &mut rng);

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
        let score = rewards[i] as f32 / visits[i] as f32;
        println!(
            "{:?}: {} ({} / {})",
            actions[i], score, rewards[i], visits[i]
        );
        if score > best_score {
            best_i = i;
            best_score = score;
        }
    }

    actions[best_i]
}

fn rollout(root_public_info: &PublicInfo, mut rng: &mut StdRng) -> (Action, f32) {
    assert!(root_public_info.player_hand.is_none());
    assert!(root_public_info.opponent_hand.is_some());

    let mut rollout_env = HanabiEnv::determinize(&root_public_info, &mut rng);
    let action = *rollout_env.actions().choose(&mut rng).unwrap();
    rollout_env.step(&action, &mut rng);

    let mut player_perspective = false;

    loop {
        if rollout_env.is_over() {
            break;
        }

        let public_info = rollout_env.public_info(player_perspective); // TODO should this be always from players perspective or just from same players?

        let step_env = HanabiEnv::determinize(&public_info, &mut rng);
        let actions = step_env.actions();
        let action = actions.choose(&mut rng).unwrap();

        rollout_env.step(action, &mut rng);

        player_perspective = !player_perspective;
    }

    (action, rollout_env.reward())
}

fn main() {
    println!("Card {}", std::mem::size_of::<Card>());
    println!("Hints {}", std::mem::size_of::<Hints>());
    println!("CardCollection {}", std::mem::size_of::<CardCollection>());
    println!("Env {}", std::mem::size_of::<HanabiEnv>());
    println!("PublicInfo {}", std::mem::size_of::<PublicInfo>());
    println!();

    let mut rng = StdRng::seed_from_u64(0);

    loop {
        let mut root_env = HanabiEnv::new(&mut rng);
        root_env.describe();
        println!();

        while !root_env.is_over() {
            let action = policy(&root_env.public_info(true), &mut rng);
            println!();
            root_env.describe();
            println!("{:?}", action);
            root_env.step(&action, &mut rng);
            root_env.describe();
            println!();
        }
        println!("{} {}", root_env.reward(), root_env.raw_score());
    }
}
