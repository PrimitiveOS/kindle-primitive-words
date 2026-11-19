# Primitive Words

Solve a five-letter puzzle in up to six attempts. Each letter is marked for an exact match, a match in another
position, or absence. Daily puzzles change on the UTC day boundary. Free play and daily results are saved separately.

The app accepts words from its independently selected English answer and guess banks. A word outside the guess bank
leaves the draft and number of attempts unchanged.

The English banks avoid a redistribution dependency on KWordle's lists. KWordle credits both English lists to
Sean Patlan, whose source repository describes them as copies of the original game's word lists and supplies no
dataset license. [KWordle credits](https://github.com/crizmo/KWordle/blob/a1f2a255ec6a69f7f916dc4e0aa382fbb7f8f742/README.md#credits),
[English list source](https://github.com/seanpatlan/wordle-words).

Run `cargo test --offline --workspace` and `cargo run --offline -p words-desktop` with the shared Tools and
Launcher repositories next to this one. The ARMHF package uses the fonts installed by Primitive Launcher.

Source is PolyForm Noncommercial 1.0.0. Documentation is CC BY-NC-SA 4.0.
