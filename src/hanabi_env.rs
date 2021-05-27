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

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Card {
    id: u8,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Hint {
    color: u8,
    suit: u8,
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
    pub total: u8,
    pub counts: [u8; 25],
}

#[derive(Copy, Clone)]
pub struct Fireworks(pub [u8; 5]);

#[derive(Clone)]
pub struct HanabiEnv {
    pub player_hand: [Card; 5],
    pub player_hints: [Hint; 5],
    pub opponent_hand: [Card; 5],
    pub opponent_hints: [Hint; 5],
    pub deck: CardCollection,
    pub discard: CardCollection,
    pub blue_tokens: u8,
    pub black_tokens: u8,
    pub fireworks: Fireworks,
    pub last_round: bool,
    pub last_round_turns_taken: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrivateInfo {
    pub opponent_hand: [Card; 5],
}

#[derive(Clone)]
pub struct PublicInfo {
    pub player_hints: [Hint; 5],
    pub opponent_hints: [Hint; 5],
    pub discard: CardCollection,
    pub blue_tokens: u8,
    pub black_tokens: u8,
    pub fireworks: Fireworks,
    pub last_round: bool,
    pub last_round_turns_taken: u8,
}

impl Color {
    fn from_id(id: u8) -> Self {
        match id {
            0 => Color::White,
            1 => Color::Red,
            2 => Color::Blue,
            3 => Color::Yellow,
            4 => Color::Green,
            _ => panic!(),
        }
    }

    fn short_name(&self) -> &str {
        match self {
            Color::White => "W",
            Color::Blue => "B",
            Color::Red => "R",
            Color::Yellow => "Y",
            Color::Green => "G",
        }
    }
}

impl Suit {
    fn from_id(id: u8) -> Self {
        match id {
            0 => Suit::One,
            1 => Suit::Two,
            2 => Suit::Three,
            3 => Suit::Four,
            4 => Suit::Five,
            _ => panic!(),
        }
    }
}

impl Card {
    fn new(color: Color, suit: Suit) -> Self {
        Card::from_parts(color as u8, suit as u8)
    }

    pub fn from_parts(color: u8, suit: u8) -> Card {
        Card {
            id: Card::parts_id(color, suit),
        }
    }

    pub fn from_id(id: u8) -> Card {
        Card { id: id }
    }

    fn none() -> Card {
        Card { id: 26 }
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn parts_id(color: u8, suit: u8) -> u8 {
        color * 5 + suit
    }

    fn is_none(&self) -> bool {
        self.id == 26
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    fn color_id(&self) -> u8 {
        self.id / 5
    }
    fn suit_id(&self) -> u8 {
        self.id % 5
    }

    fn color(&self) -> Color {
        Color::from_id(self.color_id())
    }

    fn suit(&self) -> Suit {
        Suit::from_id(self.suit_id())
    }
}

impl std::fmt::Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{}{}",
            self.color().short_name(),
            self.suit_id() + 1
        ))
    }
}

impl std::fmt::Debug for Hint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Hint(")?;
        for &color in COLORS.iter() {
            if self.matches_color(color) {
                f.write_str(color.short_name())?;
            } else {
                f.write_str(" ")?;
            }
        }
        f.write_str(",")?;
        for &suit in SUITS.iter() {
            if self.matches_suit(suit) {
                f.write_str(&(suit as u8 + 1).to_string())?;
            } else {
                f.write_str(" ")?;
            }
        }
        f.write_str(")")
    }
}

impl std::fmt::Debug for Fireworks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;
        for color_i in 0..4 {
            f.write_str(Color::from_id(color_i).short_name())?;
            if self.0[color_i as usize] == 0 {
                f.write_str(" ")?;
            } else {
                f.write_str(&self.0[color_i as usize].to_string())?;
            }
            f.write_str(" ")?;
        }
        f.write_str(Color::from_id(4).short_name())?;
        if self.0[4] == 0 {
            f.write_str(" ")?;
        } else {
            f.write_str(&self.0[4].to_string())?;
        }
        f.write_str("]")
    }
}

impl Fireworks {
    fn empty() -> Self {
        Self([0; 5])
    }

    pub fn total(&self) -> u8 {
        self.0.iter().sum::<u8>()
    }

    fn accepts(&self, card: Card) -> bool {
        self.0[card.color_id() as usize] == card.suit_id()
    }

    fn add_card(&mut self, card: Card) {
        assert!(self.accepts(card));
        self.0[card.color_id() as usize] = card.suit_id() + 1;
    }

    fn is_color_complete(&self, color: Color) -> bool {
        self.0[color as usize] == 5
    }
}

impl Hint {
    pub fn empty() -> Self {
        Self {
            color: 0b011111,
            suit: 0b011111,
        }
    }

    fn none() -> Self {
        Self {
            color: 0b100000,
            suit: 0b100000,
        }
    }

    fn is_none(&self) -> bool {
        self.color == 0b100000 || self.suit == 0b100000
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub fn set_true_color(&mut self, color: Color) {
        self.color = 1 << color as usize;
    }

    pub fn disable_color(&mut self, color: Color) {
        self.color &= !(1 << color as usize);
    }

    pub fn set_true_suit(&mut self, suit: Suit) {
        self.suit = 1 << suit as usize;
    }

    pub fn disable_suit(&mut self, suit: Suit) {
        self.suit &= !(1 << suit as usize);
    }

    fn matches_color(&self, color: Color) -> bool {
        let color_bit = 1 << color as u8;
        self.color & color_bit == color_bit
    }

    fn matches_suit(&self, suit: Suit) -> bool {
        let suit_bit = 1 << suit as u8;
        self.suit & suit_bit == suit_bit
    }

    pub fn matches(&self, card: Card) -> bool {
        self.matches_color(card.color()) && self.matches_suit(card.suit())
    }
}

impl CardCollection {
    fn empty() -> Self {
        Self {
            total: 0,
            counts: [0; 25],
        }
    }

    pub fn starting_deck() -> Self {
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

    pub fn remove_hand(&mut self, hand: &[Card; 5]) {
        for &opt_card in hand.iter() {
            if opt_card.is_some() {
                self.remove(opt_card);
            }
        }
    }

    pub fn remove_fireworks(&mut self, fireworks: &Fireworks) {
        for color_i in 0..5u8 {
            for suit_i in 0..fireworks.0[color_i as usize] {
                self.remove(Card::from_parts(color_i, suit_i));
            }
        }
    }

    pub fn subtract(&mut self, other: &Self) {
        self.total -= other.total;
        for i in 0..25 {
            self.counts[i] -= other.counts[i];
        }
    }

    fn pop<R: Rng>(&mut self, rng: &mut R) -> Card {
        if self.total > 0 {
            let card_index = rng.gen_range(0, self.total);
            let mut total = 0;
            for i in 0..25 {
                if card_index < total + self.counts[i] {
                    return self.remove(Card::from_id(i as u8));
                }
                total += self.counts[i];
            }
        }
        Card::none()
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
        let matched_card = matches.pop(&mut rng);
        if matched_card.is_some() {
            let num_matches = matches.counts[matched_card.id() as usize] + 1;
            Some((
                self.remove(matched_card),
                (num_matches as f32) / (matches.total + 1) as f32, // TODO for theory of mind, change this probabilty based on what they play
            ))
        } else {
            None
        }
    }

    fn num_of_suit(&self, suit: Suit) -> u8 {
        let mut num = 0;
        for color in 0..5 {
            num += self.counts[5 * color + suit as usize];
        }
        num
    }

    fn num_of_color(&self, color: Color) -> u8 {
        let mut num = 0;
        for suit in 0..5 {
            num += self.counts[5 * (color as usize) + suit];
        }
        num
    }
}

fn determinize_hints<R: Rng>(
    deck: &mut CardCollection,
    hints: &[Hint; 5],
    mut rng: &mut R,
) -> ([Card; 5], f32) {
    // go to first card
    let mut i = 0;
    while hints[i].is_none() && i < 5 {
        i += 1;
    }

    let mut cards = [Card::none(); 5];
    let mut prob = 1.0;
    while i < 5 {
        match deck.pop_match(&hints[i], &mut rng) {
            Some((card, p)) => {
                cards[i] = card;
                prob *= p;
            }
            None => {
                // there are no matching cards! start over
                // remove any cards we've set
                // TODO optimize this so we don't throw away good work!
                prob = 1.0;
                for j in 0..5 {
                    if cards[j].is_some() {
                        deck.add(cards[j]);
                        cards[j] = Card::none();
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
        self.discard.add(self.player_hand[i]);
        self.player_hand[i] = Card::none();
        self.player_hints[i] = Hint::none();
    }

    fn draw_into<R: Rng>(&mut self, mut rng: &mut R, i: usize) {
        let card = self.deck.pop(&mut rng);
        self.player_hand[i] = card;
        if card.is_some() {
            self.player_hints[i] = Hint::empty();
        } else {
            self.last_round = true;
            self.player_hints[i] = Hint::none();
        }
    }

    pub fn describe(&self) {
        println!(
            "Deck=|{}| Discard=|{}| Fireworks={:?} Blue={} Black={} FutureReward={}",
            self.deck.total,
            self.discard.total,
            self.fireworks,
            self.blue_tokens,
            self.black_tokens,
            possible_future_rewards(&self.fireworks, &self.discard),
        );
        println!("----- Me -----");
        println!("{:?}", self.player_hand);
        println!("{:?}", self.player_hints);
        println!("----- Op -----");
        println!("{:?}", self.opponent_hand);
        println!("{:?}", self.opponent_hints);
    }

    fn hint_matches(&self, hints: &[Hint; 5], hint: &Hint) -> Vec<usize> {
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
        let num_fireworks = self.fireworks.total();
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

fn possible_future_rewards(fireworks: &Fireworks, discard: &CardCollection) -> u8 {
    let mut cards_in_play = CardCollection::starting_deck();
    cards_in_play.subtract(discard);

    let mut future_rewards = 0;
    for color in 0..5 {
        let played_suit = fireworks.0[color];
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
        let reward = (self.fireworks.total() as f32) / 25.0;
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
            player_hints: [Hint::empty(); 5],
            opponent_hints: [Hint::empty(); 5],
            deck: deck,
            discard: CardCollection::empty(),
            blue_tokens: 8,
            black_tokens: 4,
            fireworks: Fireworks::empty(),
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
                let play = Action::Play(self.player_hints[i]);
                let discard = Action::Discard(self.player_hints[i]);
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
                    .filter(|c| c.is_some() && c.color() == color)
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
                    .filter(|c| c.is_some() && c.suit() == suit)
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
                    if self.opponent_hand[i].is_some() {
                        if self.opponent_hand[i].color() == color {
                            self.opponent_hints[i].set_true_color(color);
                        } else {
                            self.opponent_hints[i].disable_color(color);
                        }
                    }
                }
                self.blue_tokens -= 1;
            }
            &Action::SuitHint(suit) => {
                for i in 0..5 {
                    if self.opponent_hand[i].is_some() {
                        if self.opponent_hand[i].suit() == suit {
                            self.opponent_hints[i].set_true_suit(suit);
                        } else {
                            self.opponent_hints[i].disable_suit(suit);
                        }
                    }
                }
                self.blue_tokens -= 1;
            }
            &Action::Play(hint) => {
                let i = *self
                    .hint_matches(&self.player_hints, &hint)
                    .choose(&mut rng)
                    .unwrap();

                let card = self.player_hand[i];

                if self.fireworks.accepts(card) {
                    self.fireworks.add_card(card);
                    if self.fireworks.is_color_complete(card.color()) {
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
                    .hint_matches(&self.player_hints, &hint)
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
    use crate::rand::prelude::SliceRandom;
    use crate::rand::rngs::StdRng;
    use crate::rand::SeedableRng;
    #[test]
    fn test_future_reward() {
        let mut fireworks = Fireworks::empty();
        let mut discard = CardCollection::empty();

        assert_eq!(possible_future_rewards(&fireworks, &discard), 25);

        discard.add(Card::new(Color::White, Suit::One));
        discard.add(Card::new(Color::White, Suit::One));
        discard.add(Card::new(Color::White, Suit::One));

        assert_eq!(possible_future_rewards(&fireworks, &discard), 20);

        discard.add(Card::new(Color::Green, Suit::One));

        assert_eq!(possible_future_rewards(&fireworks, &discard), 20);

        discard.add(Card::new(Color::Yellow, Suit::Three));
        discard.add(Card::new(Color::Yellow, Suit::Three));

        assert_eq!(possible_future_rewards(&fireworks, &discard), 17);

        discard.add(Card::new(Color::Red, Suit::Five));

        assert_eq!(possible_future_rewards(&fireworks, &discard), 16);

        fireworks.add_card(Card::new(Color::Blue, Suit::One));
        fireworks.add_card(Card::new(Color::Blue, Suit::Two));

        assert_eq!(possible_future_rewards(&fireworks, &discard), 14);
    }

    #[test]
    fn test_weird() {
        let public_info = PublicInfo {
            player_hints: [
                Hint {
                    color: 0b11110,
                    suit: 0b00001,
                },
                Hint {
                    color: 0b00001,
                    suit: 0b01111,
                },
                Hint {
                    color: 0b00001,
                    suit: 0b10000,
                },
                Hint {
                    color: 0b11111,
                    suit: 0b01111,
                },
                Hint {
                    color: 0b11111,
                    suit: 0b01111,
                },
            ],
            opponent_hints: [
                Hint {
                    color: 0b00100,
                    suit: 0b11111,
                },
                Hint {
                    color: 0b11011,
                    suit: 0b11111,
                },
                Hint {
                    color: 0b00100,
                    suit: 0b11111,
                },
                Hint {
                    color: 0b11111,
                    suit: 0b11111,
                },
                Hint {
                    color: 0b11011,
                    suit: 0b11111,
                },
            ],
            discard: CardCollection::empty(),
            blue_tokens: 7,
            black_tokens: 2,
            fireworks: Fireworks([1, 0, 0, 0, 1]),
            last_round: false,
            last_round_turns_taken: 0,
        };
        let private_info = PrivateInfo {
            opponent_hand: [
                Card::new(Color::Blue, Suit::Three),
                Card::new(Color::Yellow, Suit::One),
                Card::new(Color::Blue, Suit::One),
                Card::new(Color::Yellow, Suit::One),
                Card::new(Color::White, Suit::One),
            ],
        };
        let mut rng = StdRng::seed_from_u64(0);
        let r = public_info.reward();
        for _ in 0..1000 {
            let (op_info, p) =
                HanabiEnv::sample_opponent_info(&public_info, &private_info, &mut rng);
            let mut env = HanabiEnv::new(&public_info, &private_info, &op_info);
            env.step(
                &Action::Play(Hint {
                    color: 0b00001,
                    suit: 0b10000,
                }),
                &mut rng,
            );
            assert!(env.is_over());
            assert_eq!(env.reward(), r);
        }
    }
}
