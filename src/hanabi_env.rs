use crate::env::{Env, HasEnd, HasReward};
use crate::rand::seq::SliceRandom;
use crate::rand::Rng;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Color {
    White = 0,
    Red = 1,
    Blue = 2,
    Yellow = 3,
    Green = 4,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Suit {
    One = 0,
    Two = 1,
    Three = 2,
    Four = 3,
    Five = 4,
}

const COLORS: [Color; 5] = [
    Color::White,
    Color::Red,
    Color::Blue,
    Color::Yellow,
    Color::Green,
];
const SUITS: [Suit; 5] = [Suit::One, Suit::Two, Suit::Three, Suit::Four, Suit::Five];

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Card {
    color: Color,
    suit: Suit,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Hint {
    color: [bool; 5],
    suit: [bool; 5],
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Action {
    ColorHint(Color),
    SuitHint(Suit),
    Discard(Hint),
    Play(Hint),
}

#[derive(Copy, Clone)]
pub struct CardCollection {
    total: u8,
    counts: [u8; 25],
}

#[derive(Clone)]
pub struct HanabiEnv {
    pub player_hand: [Option<Card>; 5],
    pub player_hints: [Option<Hint>; 5],
    pub opponent_hand: [Option<Card>; 5],
    pub opponent_hints: [Option<Hint>; 5],
    pub deck: CardCollection,
    pub discard: CardCollection,
    pub blue_tokens: u8,
    pub black_tokens: u8,
    pub fireworks: [u8; 5],
    pub last_round: bool,
    pub last_round_turns_taken: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrivateInfo {
    pub opponent_hand: [Option<Card>; 5],
}

#[derive(Clone)]
pub struct PublicInfo {
    pub player_hints: [Option<Hint>; 5],
    pub opponent_hints: [Option<Hint>; 5],
    pub discard: CardCollection,
    pub blue_tokens: u8,
    pub black_tokens: u8,
    pub fireworks: [u8; 5],
    pub last_round: bool,
    pub last_round_turns_taken: u8,
}

impl Card {
    fn id(&self) -> u8 {
        Card::parts_id(self.color as u8, self.suit as u8)
    }

    fn parts_id(color: u8, suit: u8) -> u8 {
        color * 5 + suit
    }

    fn from_parts(color: u8, suit: u8) -> Card {
        let color = match color {
            0 => Color::White,
            1 => Color::Red,
            2 => Color::Blue,
            3 => Color::Yellow,
            4 => Color::Green,
            _ => panic!(),
        };

        let suit = match suit {
            0 => Suit::One,
            1 => Suit::Two,
            2 => Suit::Three,
            3 => Suit::Four,
            4 => Suit::Five,
            _ => panic!(),
        };

        Card {
            color: color,
            suit: suit,
        }
    }

    fn from_id(id: u8) -> Card {
        let c = id / 5;
        let s = id % 5;
        Card::from_parts(c, s)
    }
}

impl std::fmt::Debug for Hint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Hint")
            .field(
                &self
                    .color
                    .iter()
                    .map(|&b| (b as u8).to_string())
                    .collect::<Vec<String>>()
                    .join(""),
            )
            .field(
                &self
                    .suit
                    .iter()
                    .map(|&b| (b as u8).to_string())
                    .collect::<Vec<String>>()
                    .join(""),
            )
            .finish()
    }
}

impl Hint {
    fn empty() -> Self {
        Self {
            color: [true; 5],
            suit: [true; 5],
        }
    }

    fn set_true_color(&mut self, color: Color) {
        for i in 0..5 {
            self.color[i] = false;
        }
        self.color[color as usize] = true;
    }

    fn disable_color(&mut self, color: Color) {
        self.color[color as usize] = false;
    }

    fn set_true_suit(&mut self, suit: Suit) {
        for i in 0..5 {
            self.suit[i] = false;
        }
        self.suit[suit as usize] = true;
    }

    fn disable_suit(&mut self, suit: Suit) {
        self.suit[suit as usize] = false;
    }

    fn matches(&self, card: Card) -> bool {
        self.color[card.color as usize] && self.suit[card.suit as usize]
    }
}

impl CardCollection {
    fn empty() -> Self {
        Self {
            total: 0,
            counts: [0; 25],
        }
    }

    fn starting_deck() -> Self {
        Self {
            total: 50,
            counts: [
                3, 2, 2, 2, 1, //  White
                3, 2, 2, 2, 1, //  Red
                3, 2, 2, 2, 1, //  Blue
                3, 2, 2, 2, 1, //  Yellow
                3, 2, 2, 2, 1, //  Green
            ],
        }
    }

    fn add(&mut self, card: Card) {
        self.total += 1;
        self.counts[card.id() as usize] += 1;
    }

    fn remove(&mut self, card: Card) -> Card {
        self.total -= 1;
        self.counts[card.id() as usize] -= 1;
        card
    }

    fn remove_hand(&mut self, hand: &[Option<Card>; 5]) {
        for opt_card in hand.iter() {
            if opt_card.is_some() {
                self.remove(opt_card.unwrap());
            }
        }
    }

    fn remove_fireworks(&mut self, fireworks: &[u8; 5]) {
        for color_i in 0..5u8 {
            for suit_i in 0..fireworks[color_i as usize] {
                self.remove(Card::from_id((color_i * 5 + suit_i) as u8));
            }
        }
    }

    fn subtract(&mut self, other: &Self) {
        self.total -= other.total;
        for i in 0..25 {
            self.counts[i] -= other.counts[i];
        }
    }

    fn pop<R: Rng>(&mut self, rng: &mut R) -> Option<Card> {
        if self.total > 0 {
            let card_index = rng.gen_range(0, self.total);
            let mut total = 0;
            for i in 0..25 {
                if card_index < total + self.counts[i] {
                    return Some(self.remove(Card::from_id(i as u8)));
                }
                total += self.counts[i];
            }
        }
        None
    }

    fn pop_match<R: Rng>(&mut self, hint: &Hint, mut rng: &mut R) -> Option<(Card, f32)> {
        let mut matches = self.clone();
        for i in 0..25 {
            let card = Card::from_id(i as u8);
            if self.counts[i] > 0 && !hint.matches(card) {
                matches.total -= matches.counts[i];
                matches.counts[i] = 0;
            }
        }
        match matches.pop(&mut rng) {
            Some(card) => Some((
                self.remove(card),
                1.0 / (matches.total + 1) as f32, // TODO for theory of mind, change this probabilty based on what they play
            )),
            None => None,
        }
    }
}

fn determinize_hints<R: Rng>(
    deck: &mut CardCollection,
    hints: &[Option<Hint>; 5],
    mut rng: &mut R,
) -> ([Option<Card>; 5], f32) {
    // go to first card
    let mut i = 0;
    while hints[i].is_none() && i < 5 {
        i += 1;
    }

    let mut cards = [None; 5];
    let mut prob = 1.0;
    while i < 5 {
        match deck.pop_match(&hints[i].unwrap(), &mut rng) {
            Some((card, p)) => {
                cards[i] = Some(card);
                prob *= p;
            }
            None => {
                // there are no matching cards! start over
                // remove any cards we've set
                // TODO optimize this so we don't throw away good work!
                prob = 1.0;
                for j in 0..5 {
                    if cards[j].is_some() {
                        deck.add(cards[j].unwrap());
                        cards[j] = None;
                    }
                }

                // go to first card
                i = 0;
                while hints[i].is_none() && i < 5 {
                    i += 1;
                }
                continue;
            }
        }

        // go to next card
        loop {
            i += 1;
            if i == 5 || hints[i].is_some() {
                break;
            }
        }
    }

    (cards, prob)
}

impl HanabiEnv {
    fn discard_at(&mut self, i: usize) {
        self.discard.add(self.player_hand[i].unwrap());
        self.player_hand[i] = None;
        self.player_hints[i] = None;
    }

    fn draw_into<R: Rng>(&mut self, mut rng: &mut R, i: usize) {
        self.player_hand[i] = self.deck.pop(&mut rng);
        if self.player_hand[i].is_none() {
            self.last_round = true;
            self.player_hints[i] = None;
        } else {
            self.player_hints[i] = Some(Hint::empty());
        }
    }

    pub fn describe(&self) {
        println!(
            "Deck=|{}| Discard=|{}| Fireworks={:?} Blue={} Black={}",
            self.deck.total,
            self.discard.total,
            self.fireworks,
            self.blue_tokens,
            self.black_tokens,
        );
        println!("----- Me -----");
        println!("{:?}", self.player_hand);
        println!("{:?}", self.player_hints);
        println!("----- Op -----");
        println!("{:?}", self.opponent_hand);
        println!("{:?}", self.opponent_hints);
    }

    fn hint_matches(&self, hints: &[Option<Hint>; 5], hint: &Option<Hint>) -> Vec<usize> {
        hints
            .iter()
            .enumerate()
            .filter(|&(_, h)| h == hint)
            .map(|(i, _)| i)
            .collect()
    }
}

impl HasEnd for PublicInfo {
    fn is_over(&self) -> bool {
        let num_player_cards = self.player_hints.iter().filter(|h| h.is_some()).count() as u8;
        let num_opponent_cards = self.opponent_hints.iter().filter(|h| h.is_some()).count() as u8;
        let num_fireworks = self.fireworks.iter().sum::<u8>();
        self.black_tokens == 1
            || num_fireworks == 25
            || (self.discard.total + num_player_cards + num_opponent_cards + num_fireworks == 50
                && self.last_round
                && self.last_round_turns_taken == 2)
    }
}

impl HasEnd for HanabiEnv {
    fn is_over(&self) -> bool {
        // self.black_tokens == 1
        //     || self.fireworks.iter().sum::<u8>() == 25
        //     || (self.deck.total == 0 && self.last_round && self.last_round_turns_taken == 2)
        self.public_info().is_over()
    }
}

fn possible_future_rewards(fireworks: &[u8; 5], discard: &CardCollection) -> u8 {
    let mut cards_in_play = CardCollection::starting_deck();
    cards_in_play.subtract(discard);

    let mut future_rewards = 0;
    for color in 0..5 {
        let played_suit = fireworks[color];
        for suit in played_suit..5 {
            if cards_in_play.counts[Card::parts_id(color as u8, suit as u8) as usize] == 0 {
                break;
            }
            future_rewards += 1;
        }
    }
    future_rewards
}

impl HasReward for PublicInfo {
    type Reward = f32;

    fn reward(&self) -> Self::Reward {
        let reward = (self.fireworks.iter().sum::<u8>() as f32) / 25.0;
        let black_tokens = (self.black_tokens as f32 - 1.0) / 3.0;
        let future_reward = possible_future_rewards(&self.fireworks, &self.discard) as f32 / 25.0;
        reward + black_tokens * future_reward
    }
}

impl HasReward for HanabiEnv {
    type Reward = f32;

    fn reward(&self) -> Self::Reward {
        self.public_info().reward()
    }
}

impl Env for HanabiEnv {
    type PublicInfo = PublicInfo;
    type PrivateInfo = PrivateInfo;
    type Action = Action;

    fn new(
        public_info: &Self::PublicInfo,
        player_private_info: &Self::PrivateInfo,
        opponent_private_info: &Self::PrivateInfo,
    ) -> Self {
        let mut deck = CardCollection::starting_deck();
        deck.subtract(&public_info.discard);
        deck.remove_fireworks(&public_info.fireworks);
        deck.remove_hand(&player_private_info.opponent_hand);
        deck.remove_hand(&opponent_private_info.opponent_hand);
        Self {
            player_hand: opponent_private_info.opponent_hand,
            player_hints: public_info.player_hints,
            opponent_hand: player_private_info.opponent_hand,
            opponent_hints: public_info.opponent_hints,
            deck: deck,
            discard: public_info.discard,
            blue_tokens: public_info.blue_tokens,
            black_tokens: public_info.black_tokens,
            fireworks: public_info.fireworks,
            last_round: public_info.last_round,
            last_round_turns_taken: public_info.last_round_turns_taken,
        }
    }

    fn random<R: Rng>(mut rng: &mut R) -> Self {
        let mut deck = CardCollection::starting_deck();

        let player_hand = [
            deck.pop(&mut rng),
            deck.pop(&mut rng),
            deck.pop(&mut rng),
            deck.pop(&mut rng),
            deck.pop(&mut rng),
        ];
        let opponent_hand = [
            deck.pop(&mut rng),
            deck.pop(&mut rng),
            deck.pop(&mut rng),
            deck.pop(&mut rng),
            deck.pop(&mut rng),
        ];

        Self {
            player_hand: player_hand,
            opponent_hand: opponent_hand,
            player_hints: [Some(Hint::empty()); 5],
            opponent_hints: [Some(Hint::empty()); 5],
            deck: deck,
            discard: CardCollection::empty(),
            blue_tokens: 8,
            black_tokens: 4,
            fireworks: [0; 5],
            last_round: false,
            last_round_turns_taken: 0,
        }
    }

    fn sample_opponent_info<R: Rng>(
        public_info: &Self::PublicInfo,
        player_private_info: &Self::PrivateInfo,
        mut rng: &mut R,
    ) -> (Self::PrivateInfo, f32) {
        let mut deck = CardCollection::starting_deck();
        deck.subtract(&public_info.discard);
        deck.remove_fireworks(&public_info.fireworks);
        deck.remove_hand(&player_private_info.opponent_hand);
        let (player_hand, prob) = determinize_hints(&mut deck, &public_info.player_hints, &mut rng);
        (
            PrivateInfo {
                opponent_hand: player_hand,
            },
            prob,
        )
    }

    fn public_info(&self) -> Self::PublicInfo {
        PublicInfo {
            player_hints: self.player_hints,
            opponent_hints: self.opponent_hints,
            discard: self.discard.clone(),
            blue_tokens: self.blue_tokens,
            black_tokens: self.black_tokens,
            fireworks: self.fireworks,
            last_round: self.last_round,
            last_round_turns_taken: self.last_round_turns_taken,
        }
    }

    fn private_info(&self, player_perspective: bool) -> Self::PrivateInfo {
        PrivateInfo {
            opponent_hand: if player_perspective {
                self.opponent_hand
            } else {
                self.player_hand
            },
        }
    }

    fn actions(&self) -> Vec<Self::Action> {
        let mut actions = Vec::new();

        // play & discard actions
        for i in 0..5 {
            if self.player_hand[i].is_some() {
                let play = Action::Play(self.player_hints[i].unwrap());
                let discard = Action::Discard(self.player_hints[i].unwrap());
                if actions.iter().position(|&a| a == play).is_none() {
                    actions.push(play);
                }
                if self.blue_tokens < 8 && actions.iter().position(|&a| a == discard).is_none() {
                    actions.push(discard);
                }
            }
        }

        if self.blue_tokens > 0 {
            // color hint actions
            for &color in COLORS.iter() {
                let num_of_color = self
                    .opponent_hand
                    .iter()
                    .filter(|c| c.is_some() && c.unwrap().color == color)
                    .count();
                if num_of_color > 0 {
                    actions.push(Action::ColorHint(color));
                }
            }

            // suit hint actions
            for &suit in SUITS.iter() {
                let num_in_suit = self
                    .opponent_hand
                    .iter()
                    .filter(|c| c.is_some() && c.unwrap().suit == suit)
                    .count();
                if num_in_suit > 0 {
                    actions.push(Action::SuitHint(suit));
                }
            }
        }

        actions
    }

    fn step<R: Rng>(&mut self, action: &Self::Action, mut rng: &mut R) {
        match action {
            &Action::ColorHint(color) => {
                for i in 0..5 {
                    if let Some(card) = self.opponent_hand[i] {
                        if card.color == color {
                            self.opponent_hints[i] = self.opponent_hints[i].map(|mut h| {
                                h.set_true_color(color);
                                h
                            });
                        } else {
                            self.opponent_hints[i] = self.opponent_hints[i].map(|mut h| {
                                h.disable_color(color);
                                h
                            });
                        }
                    }
                }
                self.blue_tokens -= 1;
            }
            &Action::SuitHint(suit) => {
                for i in 0..5 {
                    if let Some(card) = self.opponent_hand[i] {
                        if card.suit == suit {
                            self.opponent_hints[i] = self.opponent_hints[i].map(|mut h| {
                                h.set_true_suit(suit);
                                h
                            });
                        } else {
                            self.opponent_hints[i] = self.opponent_hints[i].map(|mut h| {
                                h.disable_suit(suit);
                                h
                            });
                        }
                    }
                }
                self.blue_tokens -= 1;
            }
            &Action::Play(hint) => {
                let i = *self
                    .hint_matches(&self.player_hints, &Some(hint))
                    .choose(&mut rng)
                    .unwrap();
                let card = self.player_hand[i].unwrap();
                if self.fireworks[card.color as usize] == (card.suit as u8) {
                    self.fireworks[card.color as usize] += 1;

                    if self.fireworks[card.color as usize] == 5 {
                        self.blue_tokens += 1;
                        if self.blue_tokens > 8 {
                            self.blue_tokens = 8;
                        }
                    }
                } else {
                    self.discard_at(i);
                    self.black_tokens -= 1;
                }
                self.draw_into(&mut rng, i);
            }
            &Action::Discard(hint) => {
                let i = *self
                    .hint_matches(&self.player_hints, &Some(hint))
                    .choose(&mut rng)
                    .unwrap();
                self.discard_at(i);
                self.draw_into(&mut rng, i);
                self.blue_tokens += 1;
            }
        }

        if self.last_round {
            self.last_round_turns_taken += 1;
        }

        std::mem::swap(&mut self.player_hand, &mut self.opponent_hand);
        std::mem::swap(&mut self.player_hints, &mut self.opponent_hints);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_future_reward() {
        let mut fireworks = [0; 5];
        let mut discard = CardCollection::empty();

        assert_eq!(possible_future_rewards(&fireworks, &discard), 25);

        discard.add(Card {
            color: Color::White,
            suit: Suit::One,
        });
        discard.add(Card {
            color: Color::White,
            suit: Suit::One,
        });
        discard.add(Card {
            color: Color::White,
            suit: Suit::One,
        });

        assert_eq!(possible_future_rewards(&fireworks, &discard), 20);

        discard.add(Card {
            color: Color::Green,
            suit: Suit::One,
        });

        assert_eq!(possible_future_rewards(&fireworks, &discard), 20);

        discard.add(Card {
            color: Color::Yellow,
            suit: Suit::Three,
        });
        discard.add(Card {
            color: Color::Yellow,
            suit: Suit::Three,
        });

        assert_eq!(possible_future_rewards(&fireworks, &discard), 17);

        discard.add(Card {
            color: Color::Red,
            suit: Suit::Five,
        });

        assert_eq!(possible_future_rewards(&fireworks, &discard), 16);

        fireworks[Color::Blue as usize] = 2;

        assert_eq!(possible_future_rewards(&fireworks, &discard), 14);
    }
}
