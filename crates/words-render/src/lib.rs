use std::{
    io,
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use primitive_app::{App, Change};
use primitive_ui::{
    back_button, header, screen_rect, Canvas, Input, Layout, Rect, TextStyle, CAPTION,
};
use words_core::{marks, Game, Mark, LEN, LIMIT};

const SECONDS_PER_DAY: u64 = 86_400;

pub struct View {
    pub game: Game,
    pub data: Option<PathBuf>,
    notice: String,
}

pub fn seed() -> u64 {
    timestamp().as_nanos() as u64
}

pub fn day() -> u64 {
    timestamp().as_secs() / SECONDS_PER_DAY
}

fn timestamp() -> std::time::Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
}

impl View {
    pub fn new(game: Game, data: Option<PathBuf>) -> Self {
        Self {
            game,
            data,
            notice: String::new(),
        }
    }

    fn rect(x: i32, y: i32, w: i32, h: i32) -> Rect {
        screen_rect(1264, x, y, w, h)
    }

    fn save_change(&mut self) {
        self.notice.clear();
        if let Err(error) = self.save() {
            self.notice = format!("Could not save the round. {error}");
        }
    }

    fn rollover(&mut self) -> bool {
        if self.data.is_none() || !self.game.roll_daily(day()) {
            return false;
        }
        self.save_change();
        true
    }

    fn apply(&mut self, key: primitive_ui::Key) -> Change {
        use primitive_ui::Key;
        let round = self.game.active_mut();
        if round.finished() {
            return Change::None;
        }
        self.notice.clear();
        match key {
            Key::Char(letter) if letter.is_ascii_alphabetic() && round.draft.len() < LEN => {
                round.draft.push(letter.to_ascii_lowercase())
            }
            Key::Delete if !round.draft.is_empty() => {
                round.draft.pop();
            }
            Key::Go => {
                if let Err(message) = self.game.submit() {
                    self.notice = message.into();
                    return Change::Partial;
                }
            }
            _ => return Change::None,
        }
        self.save_change();
        Change::Partial
    }

    fn draw_mark(canvas: &mut Canvas, bounds: Rect, mark: Mark) {
        let size = 15.min(bounds.w).min(bounds.h);
        let x = bounds.x + (bounds.w - size) / 2;
        let y = bounds.y + (bounds.h - size) / 2;
        let dot = Rect::new(x, y, size, size);
        match mark {
            Mark::Placed => canvas.circle(dot, 0),
            Mark::Present => {
                canvas.circle(dot, 0);
                canvas.circle(dot.inset(3), 255);
            }
            Mark::Absent => canvas.fill(Rect::new(x, y + size / 2, size, 3), 85),
        }
    }
}

impl App for View {
    fn draw(&self, canvas: &mut Canvas) {
        header(canvas, "Primitive Words", "Done");
        let round = self.game.active();
        canvas.button(
            Self::rect(40, 160, 360, 100),
            "Daily puzzle",
            !self.game.free_play,
            false,
        );
        canvas.button(
            Self::rect(422, 160, 360, 100),
            "Free play",
            self.game.free_play,
            false,
        );
        canvas.text(
            Self::rect(816, 190, 405, 70),
            &format!("Daily streak  {}", self.game.score.streak),
            CAPTION.right(),
        );
        for (i, (mark, label)) in [
            (Mark::Placed, "In place"),
            (Mark::Present, "Elsewhere"),
            (Mark::Absent, "Absent"),
        ]
        .into_iter()
        .enumerate()
        {
            let x = 40 + 250 * i as i32;
            Self::draw_mark(canvas, Self::rect(x, 275, 30, 42), mark);
            canvas.text(Self::rect(x + 32, 275, 210, 50), label, CAPTION);
        }
        for i in 0..LIMIT {
            let y = 332 + i as i32 * 89;
            let guess = round.guesses.get(i);
            let content = guess.map_or_else(
                || {
                    if i == round.guesses.len() {
                        round.draft.as_str()
                    } else {
                        ""
                    }
                },
                String::as_str,
            );
            let row_marks = guess.map(|guess| marks(&round.answer, guess));
            canvas.text(
                Self::rect(48, y + 20, 100, 58),
                &(i + 1).to_string(),
                CAPTION,
            );
            for (column, letter) in content.chars().enumerate() {
                let x = 175 + column as i32 * 175;
                canvas.text(
                    Self::rect(x, y, 90, 60),
                    &letter.to_ascii_uppercase().to_string(),
                    TextStyle::new(44).bold().center(),
                );
                if let Some(row_marks) = row_marks {
                    Self::draw_mark(canvas, Self::rect(x, y + 52, 90, 30), row_marks[column]);
                }
            }
            canvas.fill(Self::rect(160, y + 85, 1004, 2), 200);
        }
        if round.finished() {
            canvas.text(
                Self::rect(52, 902, 1160, 140),
                &format!(
                    "{}  {}",
                    if round.won() {
                        "Solved."
                    } else {
                        "The word was"
                    },
                    round.answer.to_ascii_uppercase()
                ),
                TextStyle::new(62).bold().center(),
            );
            canvas.text(
                Self::rect(52, 1050, 1160, 70),
                &format!(
                    "{} of {} daily puzzles solved · Best streak {}",
                    self.game.score.won, self.game.score.played, self.game.score.best
                ),
                CAPTION.center(),
            );
            if self.game.free_play {
                canvas.button(Self::rect(382, 1175, 500, 100), "Play again", true, false);
            }
        } else {
            canvas.text(
                Self::rect(40, 868, 1184, 73),
                if self.notice.is_empty() {
                    "Use five letters. You have six tries."
                } else {
                    &self.notice
                },
                CAPTION,
            );
            canvas.keyboard(primitive_ui::Layer::Lower, "Guess", None);
            let layout = Layout::new(1264, 1680, 4, 5);
            let keyboard_marks = round.keyboard_marks();
            for (key, rect) in layout.keyboard_keys_with(primitive_ui::Layer::Lower, "Guess") {
                let primitive_ui::Key::Char(letter) = key.key else {
                    continue;
                };
                if let Some(mark) = keyboard_marks.get(&(letter as u8)) {
                    Self::draw_mark(
                        canvas,
                        Rect::new(rect.right() - 26, rect.bottom() - 33, 21, 21),
                        *mark,
                    );
                }
            }
        }
    }

    fn input(&mut self, input: Input) -> Change {
        let changed = self.rollover();
        let redraw = if changed { Change::Full } else { Change::None };
        let Input::Tap(x, y) = input else {
            return redraw;
        };
        if back_button(1264).contains(x, y) {
            return Change::Exit;
        }
        if Self::rect(40, 160, 360, 100).contains(x, y) {
            if !self.game.free_play {
                return redraw;
            }
            self.game.free_play = false;
            self.save_change();
            return Change::Full;
        }
        if Self::rect(422, 160, 360, 100).contains(x, y) {
            if self.game.free_play {
                return redraw;
            }
            self.game.free_play = true;
            self.save_change();
            return Change::Full;
        }
        if self.game.free_play
            && self.game.free.finished()
            && Self::rect(382, 1175, 500, 100).contains(x, y)
        {
            self.game.free_round(seed());
            self.save_change();
            return Change::Full;
        }
        let layout = Layout::new(1264, 1680, 4, 5);
        let editor = primitive_ui::Editor::new(LEN);
        let action = primitive_ui::keyboard_key(&layout, &editor, "Guess", x, y)
            .map_or(Change::None, |key| self.apply(key));
        if changed {
            Change::Full
        } else {
            action
        }
    }

    fn deadline(&self) -> Option<Instant> {
        self.data.as_ref()?;
        let now = timestamp();
        let seconds = SECONDS_PER_DAY - now.as_secs() % SECONDS_PER_DAY;
        let remaining =
            Duration::from_secs(seconds) - Duration::from_nanos(now.subsec_nanos().into());
        Some(Instant::now() + remaining)
    }

    fn tick(&mut self, _now: Instant) -> Change {
        if self.rollover() {
            Change::Full
        } else {
            Change::None
        }
    }

    fn save(&mut self) -> io::Result<()> {
        if let Some(path) = &self.data {
            primitive_app::save(path, &self.game.encoded())?;
        }
        Ok(())
    }
}
