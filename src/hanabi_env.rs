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
pub struct Card(Color, Suit);

impl Card {
    fn id(&self) -> u8 {
        self.0 as u8 * 5 + self.1 as u8
    }

    fn from_id(id: u8) -> Card {
        let c = id / 5;
        let s = id % 5;
        let color = match c {
            0 => Color::White,
            1 => Color::Red,
            2 => Color::Blue,
            3 => Color::Yellow,
            4 => Color::Green,
            _ => panic!(),
        };

        let suit = match s {
            0 => Suit::One,
            1 => Suit::Two,
            2 => Suit::Three,
            3 => Suit::Four,
            4 => Suit::Five,
            _ => panic!(),
        };

        Card(color, suit)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Hint(Option<Color>, Option<Suit>);

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

    fn num_cards_matching(&self, hint: Hint) -> u8 {
        match hint {
            Hint(Some(color), Some(suit)) => self.counts[Card(color, suit).id() as usize],
            Hint(Some(color), None) => self.total_of_color(color),
            Hint(None, Some(suit)) => self.total_of_suit(suit),
            Hint(None, None) => self.total,
        }
    }

    fn pop_match<R: Rng>(&mut self, hint: Hint, mut rng: &mut R) -> Option<Card> {
        match hint {
            Hint(Some(color), Some(suit)) => Some(self.remove(Card(color, suit))),
            Hint(Some(color), None) => {
                let card_index = rng.gen_range(0, self.total_of_color(color));
                let mut total = 0;
                for i in 0..5 {
                    let ci = 5 * (color as usize) + i;
                    if card_index < total + self.counts[ci] {
                        return Some(self.remove(Card::from_id(ci as u8)));
                    }
                    total += self.counts[ci];
                }
                None
            }
            Hint(None, Some(suit)) => {
                let card_index = rng.gen_range(0, self.total_of_suit(suit));
                let mut total = 0;
                for i in 0..5 {
                    let ci = 5 * i + suit as usize;
                    if card_index < total + self.counts[ci] {
                        return Some(self.remove(Card::from_id(ci as u8)));
                    }
                    total += self.counts[ci];
                }
                None
            }
            Hint(None, None) => self.pop(&mut rng),
        }
    }

    fn total_of_color(&self, color: Color) -> u8 {
        (0..5).map(|i| self.counts[5 * (color as usize) + i]).sum()
    }

    fn total_of_suit(&self, suit: Suit) -> u8 {
        (0..5).map(|i| self.counts[5 * i + suit as usize]).sum()
    }
}

fn determinize_hints<R: Rng>(
    deck: &mut CardCollection,
    hints: [Option<Hint>; 5],
    mut rng: &mut R,
) -> [Option<Card>; 5] {
    // go to first card
    let mut i = 0;
    while hints[i].is_none() && i < 5 {
        i += 1;
    }

    let mut cards = [None; 5];
    while i < 5 {
        // if we've hit a point where there are no matching cards, start over
        // TODO optimize this so we don't throw away good work!
        if hints[i].is_some() && deck.num_cards_matching(hints[i].unwrap()) == 0 {
            // remove any cards we've set
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
        }

        // pop a card
        cards[i] = deck.pop_match(hints[i].unwrap(), &mut rng);

        // go to next card
        loop {
            i += 1;
            if i == 5 || hints[i].is_some() {
                break;
            }
        }
    }
    cards
}

#[derive(Clone)]
pub struct HanabiEnv {
    pub player_hand: [Option<Card>; 5],
    pub opponent_hand: [Option<Card>; 5],
    pub player_hints: [Option<Hint>; 5],
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

impl HanabiEnv {
    pub fn new<R: Rng>(mut rng: &mut R) -> Self {
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
            player_hints: [Some(Hint(None, None)); 5],
            opponent_hand: opponent_hand,
            opponent_hints: [Some(Hint(None, None)); 5],
            deck: deck,
            discard: CardCollection::empty(),
            blue_tokens: 8,
            black_tokens: 4,
            fireworks: [0; 5],
            last_round: false,
            last_round_turns_taken: 0,
        }
    }

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
            self.player_hints[i] = Some(Hint(None, None));
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

    fn hint_matches(&self, hints: &[Option<Hint>; 5], hint: Hint) -> Vec<usize> {
        hints
            .iter()
            .enumerate()
            .filter(|(_, h)| h.is_some() && h.unwrap() == hint)
            .map(|(i, _)| i)
            .collect()
    }
}

impl HasEnd for HanabiEnv {
    fn is_over(&self) -> bool {
        self.black_tokens == 1
            || self.fireworks.iter().sum::<u8>() == 25
            || (self.deck.total == 0 && self.last_round && self.last_round_turns_taken == 2)
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

impl HasReward for PublicInfo {
    type Reward = f32;

    fn reward(&self) -> Self::Reward {
        let reward = (self.fireworks.iter().sum::<u8>() as f32) / 25.0;
        let black_tokens = (self.black_tokens as f32 - 1.0) / 3.0;
        reward + black_tokens
    }
}

impl HasReward for HanabiEnv {
    type Reward = f32;

    fn reward(&self) -> Self::Reward {
        let reward = (self.fireworks.iter().sum::<u8>() as f32) / 25.0;
        let black_tokens = (self.black_tokens as f32 - 1.0) / 3.0;
        reward + black_tokens
    }
}

impl Env for HanabiEnv {
    type PublicInfo = PublicInfo;
    type PrivateInfo = PrivateInfo;
    type Action = Action;

    fn private_info(&self, player_perspective: bool) -> Self::PrivateInfo {
        PrivateInfo {
            opponent_hand: if player_perspective {
                self.opponent_hand
            } else {
                self.player_hand
            },
        }
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

    fn determinize<R: Rng>(
        public_info: &Self::PublicInfo,
        private_info: &Self::PrivateInfo,
        mut rng: &mut R,
    ) -> Self {
        let mut deck = CardCollection::starting_deck();
        deck.subtract(&public_info.discard);

        for color_i in 0..5u8 {
            for suit_i in 0..public_info.fireworks[color_i as usize] {
                deck.remove(Card::from_id((color_i * 5 + suit_i) as u8));
            }
        }

        for opt_card in private_info.opponent_hand.iter() {
            if opt_card.is_some() {
                deck.remove(opt_card.unwrap());
            }
        }

        let player_hand = determinize_hints(&mut deck, public_info.player_hints, &mut rng);

        Self {
            player_hand: player_hand,
            player_hints: public_info.player_hints,
            opponent_hand: private_info.opponent_hand,
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
                    .filter(|c| c.is_some() && c.unwrap().0 == color)
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
                    .filter(|c| c.is_some() && c.unwrap().1 == suit)
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
                    if self.opponent_hand[i].is_some() && self.opponent_hand[i].unwrap().0 == color
                    {
                        self.opponent_hints[i] = self.opponent_hints[i].map(|mut h| {
                            h.0 = Some(color);
                            h
                        });
                    }
                }
                self.blue_tokens -= 1;
            }
            &Action::SuitHint(suit) => {
                for i in 0..5 {
                    if self.opponent_hand[i].is_some() && self.opponent_hand[i].unwrap().1 == suit {
                        self.opponent_hints[i] = self.opponent_hints[i].map(|mut h| {
                            h.1 = Some(suit);
                            h
                        });
                    }
                }
                self.blue_tokens -= 1;
            }
            &Action::Play(hint) => {
                let i = *self
                    .hint_matches(&self.player_hints, hint)
                    .choose(&mut rng)
                    .unwrap();
                let card = self.player_hand[i].unwrap();
                if self.fireworks[card.0 as usize] == (card.1 as u8) {
                    self.fireworks[card.0 as usize] += 1;

                    if self.fireworks[card.0 as usize] == 5 {
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
                    .hint_matches(&self.player_hints, hint)
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
