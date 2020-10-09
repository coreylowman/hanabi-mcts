use crate::env::Env;
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

#[derive(Copy, Clone, Debug)]
pub struct Card(Color, Suit);

impl Card {
    fn color(&self) -> Color {
        self.0
    }

    fn suit(&self) -> Suit {
        self.1
    }

    fn id(&self) -> u8 {
        self.color() as u8 * 5 + self.suit() as u8
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

#[derive(Copy, Clone, Debug)]
pub struct CardStatus(Option<Color>, Option<Suit>);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Action {
    ColorHint(Color),
    SuitHint(Suit),
    Discard(usize),
    Play(usize),
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

    fn num_cards_matching(&self, card_status: CardStatus) -> u8 {
        match card_status {
            CardStatus(Some(color), Some(suit)) => self.counts[Card(color, suit).id() as usize],
            CardStatus(Some(color), None) => self.total_of_color(color),
            CardStatus(None, Some(suit)) => self.total_of_suit(suit),
            CardStatus(None, None) => self.total,
        }
    }

    fn pop_match<R: Rng>(&mut self, card_status: CardStatus, mut rng: &mut R) -> Option<Card> {
        match card_status {
            CardStatus(Some(color), Some(suit)) => Some(self.remove(Card(color, suit))),
            CardStatus(Some(color), None) => {
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
            CardStatus(None, Some(suit)) => {
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
            CardStatus(None, None) => self.pop(&mut rng),
        }
    }

    fn total_of_color(&self, color: Color) -> u8 {
        (0..5).map(|i| self.counts[5 * (color as usize) + i]).sum()
    }

    fn total_of_suit(&self, suit: Suit) -> u8 {
        (0..5).map(|i| self.counts[5 * i + suit as usize]).sum()
    }
}

#[derive(Clone)]
pub struct HanabiEnv {
    pub player_hand: [Option<Card>; 5],
    pub opponent_hand: [Option<Card>; 5],
    pub player_hand_status: [CardStatus; 5],
    pub opponent_hand_status: [CardStatus; 5],
    pub deck: CardCollection,
    pub discard: CardCollection,
    pub blue_tokens: u8,
    pub black_tokens: u8,
    pub fireworks: [u8; 5],
    pub last_round: bool,
    pub last_round_turns_taken: u8,
}

pub struct PublicInfo {
    pub player_hand: Option<[Option<Card>; 5]>,
    pub player_hand_status: [CardStatus; 5],
    pub opponent_hand: Option<[Option<Card>; 5]>,
    pub opponent_hand_status: [CardStatus; 5],
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
            player_hand_status: [CardStatus(None, None); 5],
            opponent_hand: opponent_hand,
            opponent_hand_status: [CardStatus(None, None); 5],
            deck: deck,
            discard: CardCollection::empty(),
            blue_tokens: 8,
            black_tokens: 4,
            fireworks: [0; 5],
            last_round: false,
            last_round_turns_taken: 0,
        }
    }

    pub fn raw_score(&self) -> u8 {
        self.fireworks[0]
            + self.fireworks[1]
            + self.fireworks[2]
            + self.fireworks[3]
            + self.fireworks[4]
    }

    fn discard_at(&mut self, i: usize) {
        self.discard.add(self.player_hand[i].unwrap());
        self.player_hand[i] = None;
        self.player_hand_status[i] = CardStatus(None, None);
    }

    fn draw_into<R: Rng>(&mut self, mut rng: &mut R, i: usize) {
        self.player_hand[i] = self.deck.pop(&mut rng);
        if self.player_hand[i].is_none() {
            self.last_round = true;
        }
        self.player_hand_status[i] = CardStatus(None, None);
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
        println!("{:?}", self.player_hand_status);
        println!("----- Op -----");
        println!("{:?}", self.opponent_hand);
        println!("{:?}", self.opponent_hand_status);
    }
}

impl Env for HanabiEnv {
    type PublicInfo = PublicInfo;
    type Action = Action;
    type Reward = f32;

    fn is_over(&self) -> bool {
        self.black_tokens == 1
            || self.raw_score() == 25
            || (self.deck.total == 0 && self.last_round && self.last_round_turns_taken == 2)
    }

    fn reward(&self) -> Self::Reward {
        //let num_used_black = 4.0 - self.black_tokens as f32;
        self.raw_score() as f32
        //- (num_used_black * 6.25) / 25.0
        // TODO incoporate number of black_tokens used as a negative reward
    }

    fn public_info(&self, player_perspective: bool) -> Self::PublicInfo {
        PublicInfo {
            player_hand: if player_perspective {
                None
            } else {
                Some(self.player_hand)
            },
            opponent_hand: if player_perspective {
                Some(self.opponent_hand)
            } else {
                None
            },
            player_hand_status: self.player_hand_status,
            opponent_hand_status: self.opponent_hand_status,
            discard: self.discard.clone(),
            blue_tokens: self.blue_tokens,
            black_tokens: self.black_tokens,
            fireworks: self.fireworks,
            last_round: self.last_round,
            last_round_turns_taken: self.last_round_turns_taken,
        }
    }

    fn determinize<R: Rng>(public_info: &Self::PublicInfo, mut rng: &mut R) -> Self {
        let mut deck = CardCollection::starting_deck();
        deck.subtract(&public_info.discard);

        for color_i in 0..5u8 {
            for suit_i in 0..public_info.fireworks[color_i as usize] {
                deck.remove(Card::from_id((color_i * 5 + suit_i) as u8));
            }
        }

        let (player_hand, opponent_hand) = if public_info.player_hand.is_some() {
            for opt_card in public_info.player_hand.unwrap().iter() {
                if opt_card.is_some() {
                    deck.remove(opt_card.unwrap());
                }
            }
            let mut status = [
                Some(public_info.opponent_hand_status[0]),
                Some(public_info.opponent_hand_status[1]),
                Some(public_info.opponent_hand_status[2]),
                Some(public_info.opponent_hand_status[3]),
                Some(public_info.opponent_hand_status[4]),
            ];
            let mut cards = [None; 5];
            for _ in 0..5 {
                let (i, _) = status
                    .iter()
                    .enumerate()
                    .filter(|(_, os)| os.is_some())
                    .map(|(i, &s)| (i, deck.num_cards_matching(s.unwrap())))
                    .min_by_key(|&(_i, n)| n)
                    .unwrap();
                cards[i] = deck.pop_match(status[i].unwrap(), &mut rng);
                status[i] = None;
            }

            let mut player_hand = public_info.player_hand.unwrap();
            // TODO shuffle hand & status?
            // player_hand.shuffle(&mut rng);
            (
                player_hand,
                [cards[0], cards[1], cards[2], cards[3], cards[4]],
            )
        } else {
            for opt_card in public_info.opponent_hand.unwrap().iter() {
                if opt_card.is_some() {
                    deck.remove(opt_card.unwrap());
                }
            }
            let mut status = [
                Some(public_info.player_hand_status[0]),
                Some(public_info.player_hand_status[1]),
                Some(public_info.player_hand_status[2]),
                Some(public_info.player_hand_status[3]),
                Some(public_info.player_hand_status[4]),
            ];
            let mut cards = [None; 5];
            for _ in 0..5 {
                let (i, _) = status
                    .iter()
                    .enumerate()
                    .filter(|(_, os)| os.is_some())
                    .map(|(i, &s)| (i, deck.num_cards_matching(s.unwrap())))
                    .min_by_key(|&(_i, n)| n)
                    .unwrap();
                cards[i] = deck.pop_match(status[i].unwrap(), &mut rng);
                status[i] = None;
            }

            let mut opponent_hand = public_info.opponent_hand.unwrap();
            // TODO shuffle hand & status?
            // opponent_hand.shuffle(&mut rng);
            (
                [cards[0], cards[1], cards[2], cards[3], cards[4]],
                opponent_hand,
            )
        };

        Self {
            player_hand: player_hand,
            player_hand_status: public_info.player_hand_status,
            opponent_hand: opponent_hand,
            opponent_hand_status: public_info.opponent_hand_status,
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
                actions.push(Action::Play(i));
                if self.blue_tokens < 8 {
                    actions.push(Action::Discard(i));
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
                        self.opponent_hand_status[i].0 = Some(color);
                    }
                }
                self.blue_tokens -= 1;
            }
            &Action::SuitHint(suit) => {
                for i in 0..5 {
                    if self.opponent_hand[i].is_some() && self.opponent_hand[i].unwrap().1 == suit {
                        self.opponent_hand_status[i].1 = Some(suit);
                    }
                }
                self.blue_tokens -= 1;
            }
            &Action::Play(i) => {
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
            &Action::Discard(i) => {
                self.discard_at(i);
                self.draw_into(&mut rng, i);
                self.blue_tokens += 1;
            }
        }

        if self.last_round {
            self.last_round_turns_taken += 1;
        }

        std::mem::swap(&mut self.player_hand, &mut self.opponent_hand);
        std::mem::swap(&mut self.player_hand_status, &mut self.opponent_hand_status);
    }
}
