#![forbid(unsafe_code)]

////////////////////////////////////////////////////////////////////////////////

use std::boxed::Box;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundOutcome {
    BothCooperated,
    LeftCheated,
    RightCheated,
    BothCheated,
}

pub struct Game {
    left: Box<dyn Agent>,
    right: Box<dyn Agent>,
    left_score: i32,
    right_score: i32,
}

impl Game {
    pub fn new(left: Box<dyn Agent>, right: Box<dyn Agent>) -> Self {
        Self {
            left,
            right,
            left_score: 0,
            right_score: 0,
        }
    }

    pub fn left_score(&self) -> i32 {
        self.left_score
    }

    pub fn right_score(&self) -> i32 {
        self.right_score
    }

    pub fn play_round(&mut self) -> RoundOutcome {
        let left_play = self.left.play();
        let right_play = self.right.play();
        self.left.fetch_play(right_play);
        self.right.fetch_play(left_play);
        if left_play && right_play {
            self.left_score += 2;
            self.right_score += 2;
            return RoundOutcome::BothCooperated;
        }
        if !left_play && right_play {
            self.left_score += 3;
            self.right_score -= 1;
            return RoundOutcome::LeftCheated;
        }
        if left_play && !right_play {
            self.left_score -= 1;
            self.right_score += 3;
            return RoundOutcome::RightCheated;
        }
        RoundOutcome::BothCheated
    }
}

////////////////////////////////////////////////////////////////////////////////

pub trait Agent {
    fn play(&self) -> bool;
    fn fetch_play(&mut self, play: bool);
}

////////////////////////////////////////////////////////////////////////////////

pub struct CheatingAgent {}

impl CheatingAgent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Agent for CheatingAgent {
    fn play(&self) -> bool {
        false
    }

    fn fetch_play(&mut self, _play: bool) {}
}

impl Default for CheatingAgent {
    fn default() -> Self {
        Self::new()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct CooperatingAgent {}

impl CooperatingAgent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Agent for CooperatingAgent {
    fn play(&self) -> bool {
        true
    }

    fn fetch_play(&mut self, _play: bool) {}
}

impl Default for CooperatingAgent {
    fn default() -> Self {
        Self::new()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct GrudgerAgent {
    got_cheated: bool,
}

impl GrudgerAgent {
    pub fn new() -> Self {
        Self { got_cheated: false }
    }
}

impl Agent for GrudgerAgent {
    fn play(&self) -> bool {
        !self.got_cheated
    }

    fn fetch_play(&mut self, play: bool) {
        self.got_cheated |= !play;
    }
}

impl Default for GrudgerAgent {
    fn default() -> Self {
        Self::new()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct CopycatAgent {
    recorded_turn: bool,
}

impl CopycatAgent {
    pub fn new() -> Self {
        Self {
            recorded_turn: true,
        }
    }
}

impl Agent for CopycatAgent {
    fn play(&self) -> bool {
        self.recorded_turn
    }

    fn fetch_play(&mut self, play: bool) {
        self.recorded_turn = play
    }
}

impl Default for CopycatAgent {
    fn default() -> Self {
        Self::new()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct DetectiveAgent {
    turn_number: i32,
    got_cheated: bool,
    recorded_turn: bool,
}

impl DetectiveAgent {
    pub fn new() -> Self {
        Self {
            turn_number: 0,
            got_cheated: false,
            recorded_turn: true,
        }
    }
}

impl Agent for DetectiveAgent {
    fn play(&self) -> bool {
        if self.turn_number == 1 {
            return false;
        }
        if self.turn_number < 4 {
            return true;
        }
        if self.got_cheated {
            return self.recorded_turn;
        }
        false
    }

    fn fetch_play(&mut self, play: bool) {
        if self.turn_number < 4 {
            self.got_cheated |= !play;
        }
        self.recorded_turn = play;
        self.turn_number += 1;
    }
}

impl Default for DetectiveAgent {
    fn default() -> Self {
        Self::new()
    }
}
