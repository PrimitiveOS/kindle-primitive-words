use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const WORDS: &str = include_str!("answers.txt");
const GUESSES: &str = include_str!("guesses.txt");
pub const LIMIT: usize = 6;
pub const LEN: usize = 5;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mark {
    #[default]
    Absent,
    Present,
    Placed,
}

pub fn marks(answer: &str, guess: &str) -> [Mark; LEN] {
    let mut marked = [Mark::Absent; LEN];
    let answer: Vec<_> = answer.bytes().take(LEN).collect();
    let guess: Vec<_> = guess.bytes().take(LEN).collect();
    let mut remaining = BTreeMap::<u8, u8>::new();
    for (i, (&a, &g)) in answer.iter().zip(&guess).enumerate() {
        if a == g {
            marked[i] = Mark::Placed;
        } else {
            *remaining.entry(a).or_default() += 1;
        }
    }
    for (i, &g) in guess.iter().enumerate() {
        if marked[i] != Mark::Placed {
            if let Some(count) = remaining.get_mut(&g).filter(|count| **count != 0) {
                *count -= 1;
                marked[i] = Mark::Present;
            }
        }
    }
    marked
}

pub fn valid(word: &str) -> bool {
    word.len() == LEN && word.bytes().all(|c| c.is_ascii_lowercase())
}

fn answer(seed: u64) -> String {
    let words: Vec<_> = WORDS
        .split_whitespace()
        .filter(|word| valid(word))
        .collect();
    let random = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let random = (random ^ (random >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    let random = (random ^ (random >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    let index = ((random ^ (random >> 31)) % words.len() as u64) as usize;
    words[index].to_owned()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Round {
    pub answer: String,
    pub guesses: Vec<String>,
    pub draft: String,
}

impl Round {
    pub fn new(answer: String) -> Self {
        Self {
            answer,
            guesses: Vec::new(),
            draft: String::new(),
        }
    }

    pub fn won(&self) -> bool {
        self.guesses.last() == Some(&self.answer)
    }

    pub fn finished(&self) -> bool {
        self.won() || self.guesses.len() >= LIMIT
    }

    pub fn submit(&mut self) -> Result<(), &'static str> {
        if self.finished() {
            return Err("This round is finished.");
        }
        if !valid(&self.draft) {
            return Err("Enter five letters first.");
        }
        if self.draft != self.answer
            && !WORDS
                .split_whitespace()
                .chain(GUESSES.split_whitespace())
                .any(|word| word == self.draft)
        {
            return Err("That word isn't in the guess list.");
        }
        self.guesses.push(std::mem::take(&mut self.draft));
        Ok(())
    }

    pub fn keyboard_marks(&self) -> BTreeMap<u8, Mark> {
        let mut result: BTreeMap<u8, Mark> = BTreeMap::new();
        for guess in &self.guesses {
            for (letter, mark) in guess.bytes().zip(marks(&self.answer, guess)) {
                result
                    .entry(letter)
                    .and_modify(|previous| *previous = (*previous).max(mark))
                    .or_insert(mark);
            }
        }
        result
    }

    fn is_valid(&self) -> bool {
        valid(&self.answer)
            && self.guesses.len() <= LIMIT
            && self.guesses.iter().all(|guess| valid(guess))
            && self.draft.len() <= LEN
            && self.draft.bytes().all(|c| c.is_ascii_lowercase())
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Score {
    pub played: u32,
    pub won: u32,
    pub streak: u32,
    pub best: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Game {
    day: u64,
    last_daily_win: Option<u64>,
    pub daily: Round,
    pub free: Round,
    pub free_play: bool,
    pub score: Score,
}

impl Game {
    pub fn restore(raw: &[u8], day: u64, seed: u64) -> Self {
        let mut game = serde_json::from_slice::<Self>(raw)
            .ok()
            .filter(|g| g.daily.is_valid() && g.free.is_valid())
            .unwrap_or_else(|| Self {
                day,
                last_daily_win: None,
                daily: Round::new(answer(day)),
                free: Round::new(answer(seed)),
                free_play: false,
                score: Score::default(),
            });
        game.roll_daily(day);
        game
    }

    pub fn roll_daily(&mut self, day: u64) -> bool {
        let date_changed = self.day != day;
        if date_changed {
            self.day = day;
            self.daily = Round::new(answer(day));
        }
        let old_streak = self.score.streak;
        if self
            .last_daily_win
            .is_none_or(|win| day.saturating_sub(win) > 1)
        {
            self.score.streak = 0;
        }
        date_changed || self.score.streak != old_streak
    }

    pub fn encoded(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("game state consists of serializable fields")
    }

    pub fn active(&self) -> &Round {
        if self.free_play {
            &self.free
        } else {
            &self.daily
        }
    }

    pub fn active_mut(&mut self) -> &mut Round {
        if self.free_play {
            &mut self.free
        } else {
            &mut self.daily
        }
    }

    pub fn free_round(&mut self, seed: u64) {
        self.free_play = true;
        let mut next = answer(seed);
        if next == self.free.answer {
            next = answer(seed.wrapping_add(1));
        }
        self.free = Round::new(next);
    }

    pub fn submit(&mut self) -> Result<(), &'static str> {
        self.active_mut().submit()?;
        if !self.free_play && self.daily.finished() {
            self.score.played = self.score.played.saturating_add(1);
            if self.daily.won() {
                self.score.won = self.score.won.saturating_add(1);
                self.score.streak = if self.last_daily_win == self.day.checked_sub(1) {
                    self.score.streak.saturating_add(1)
                } else {
                    1
                };
                self.score.best = self.score.best.max(self.score.streak);
                self.last_daily_win = Some(self.day);
            } else {
                self.score.streak = 0;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_marks_cannot_use_an_answer_letter_twice() {
        assert_eq!(
            marks("amber", "array"),
            [
                Mark::Placed,
                Mark::Present,
                Mark::Absent,
                Mark::Absent,
                Mark::Absent
            ]
        );
    }

    #[test]
    fn records_one_result_per_calendar_day_and_preserves_free_play() {
        let mut game = Game::restore(b"", 12, 33);
        game.daily.draft = game.daily.answer.clone();
        game.submit().unwrap();
        assert!(game.submit().is_err());
        game.free_round(55);
        let free_answer = game.free.answer.clone();
        let mut next = Game::restore(&game.encoded(), 13, 99);
        assert_eq!(next.score.played, 1);
        assert_eq!(next.free.answer, free_answer);
        next.free_play = false;
        next.daily.draft = next.daily.answer.clone();
        next.submit().unwrap();
        assert_eq!(next.score.streak, 2);
    }

    #[test]
    fn unknown_words_leave_the_attempt_count_and_draft_unchanged() {
        let mut round = Round::new("amber".into());
        round.draft = "zzzzz".into();
        assert_eq!(round.submit(), Err("That word isn't in the guess list."));
        assert!(round.guesses.is_empty());
        assert_eq!(round.draft, "zzzzz");
        round.draft = "adieu".into();
        round.submit().unwrap();
        assert_eq!(round.guesses, ["adieu"]);
    }

    #[test]
    fn rolls_an_open_game_into_the_new_day_without_resetting_free_play() {
        let mut game = Game::restore(b"", 12, 33);
        game.daily.draft = game.daily.answer.clone();
        game.submit().unwrap();
        game.free_round(55);
        game.free.draft = "time".into();
        assert!(!game.roll_daily(12));
        assert!(game.roll_daily(13));
        assert!(game.daily.guesses.is_empty());
        assert!(game.free_play);
        assert_eq!(game.free.draft, "time");
        assert_eq!(game.score.played, 1);
        assert_eq!(game.score.streak, 1);
        assert!(game.roll_daily(14));
        assert_eq!(game.score.streak, 0);
    }
}
