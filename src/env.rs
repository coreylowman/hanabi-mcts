use crate::rand::Rng;

pub trait Env {
    type PublicInfo;
    type Action;
    type Reward;

    fn is_over(&self) -> bool;
    fn reward(&self) -> Self::Reward;
    fn public_info(&self, player_perspective: bool) -> Self::PublicInfo;
    fn determinize<R: Rng>(public_info: &Self::PublicInfo, rng: &mut R) -> Self;
    fn actions(&self) -> Vec<Self::Action>;
    fn step<R: Rng>(&mut self, action: &Self::Action, rng: &mut R);
}
