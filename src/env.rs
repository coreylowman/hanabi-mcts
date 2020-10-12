use crate::rand::Rng;

pub trait HasEnd {
    fn is_over(&self) -> bool;
}

pub trait HasReward {
    type Reward;
    fn reward(&self) -> Self::Reward;
}

pub trait Env: HasEnd + HasReward {
    type PublicInfo: HasEnd + HasReward + Clone;
    type PrivateInfo: Clone;
    type Action;

    fn determinize<R: Rng>(
        public_info: &Self::PublicInfo,
        private_info: &Self::PrivateInfo,
        rng: &mut R,
    ) -> Self;

    fn public_info(&self) -> Self::PublicInfo;
    fn private_info(&self, player_perspective: bool) -> Self::PrivateInfo;

    fn actions(&self) -> Vec<Self::Action>;
    fn step<R: Rng>(&mut self, action: &Self::Action, rng: &mut R);
}
