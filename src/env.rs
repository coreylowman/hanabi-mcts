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

    fn random<R: Rng>(rng: &mut R) -> Self;

    fn new(
        public_info: &Self::PublicInfo,
        player_private_info: &Self::PrivateInfo,
        opponent_private_info: &Self::PrivateInfo,
    ) -> Self;

    fn sample_opponent_info<R: Rng>(
        public_info: &Self::PublicInfo,
        player_private_info: &Self::PrivateInfo,
        rng: &mut R,
    ) -> (Self::PrivateInfo, f32);

    fn public_info(&self) -> Self::PublicInfo;
    fn private_info(&self, player_perspective: bool) -> Self::PrivateInfo;

    fn actions(&self) -> Vec<Self::Action>;
    fn step<R: Rng>(&mut self, action: &Self::Action, rng: &mut R);

    fn determinize<R: Rng>(
        public_info: &Self::PublicInfo,
        player_private_info: &Self::PrivateInfo,
        mut rng: &mut R,
    ) -> (Self, f32)
    where
        Self: std::marker::Sized,
    {
        let (opponent_private_info, probability) =
            Self::sample_opponent_info(public_info, player_private_info, &mut rng);
        let env = Self::new(public_info, player_private_info, &opponent_private_info);

        (env, probability)
    }
}
