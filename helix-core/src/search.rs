use crate::movement::Direction;
use helix_stdx::rope::RopeSliceExt;

use crate::{
    graphemes::{
        nth_next_folded_grapheme_boundary, nth_next_grapheme_boundary,
        nth_prev_folded_grapheme_boundary, nth_prev_grapheme_boundary,
    },
    text_folding::{ropex::RopeSliceFoldExt, FoldAnnotations},
    RopeSlice,
};

pub trait GraphemeMatcher {
    fn grapheme_match(&self, g: RopeSlice) -> bool;
}

impl GraphemeMatcher for char {
    fn grapheme_match(&self, g: RopeSlice) -> bool {
        g == RopeSlice::from(self.encode_utf8(&mut [0; 4]) as &str)
    }
}

impl<F: Fn(RopeSlice) -> bool> GraphemeMatcher for F {
    fn grapheme_match(&self, g: RopeSlice) -> bool {
        (*self)(g)
    }
}

// Finds the positions of the nth matching character in the given direction
// starting from the pos gap-index (see Range struct for explanation).
pub fn find_nth_char(
    n: usize,
    text: RopeSlice,
    ch: char,
    pos: usize,
    direction: Direction,
) -> Option<usize> {
    match direction {
        Direction::Forward => find_nth_next(text, ch, pos, n),
        Direction::Backward => find_nth_prev(text, ch, pos, n),
    }
}

pub fn find_nth_next(
    text: RopeSlice,
    matcher: impl GraphemeMatcher,
    pos: usize,
    mut n: usize,
) -> Option<usize> {
    if n == 0 || pos > text.len_chars() {
        return None;
    }

    let mut count = 0;
    for (i, g) in text.graphemes_at(pos).skip(1).enumerate() {
        if matcher.grapheme_match(g) {
            count = i + 1;
            n -= 1;
            if n == 0 {
                break;
            }
        }
    }

    (n == 0).then(|| nth_next_grapheme_boundary(text, pos, count))
}

pub fn find_nth_prev(
    text: RopeSlice,
    matcher: impl GraphemeMatcher,
    pos: usize,
    mut n: usize,
) -> Option<usize> {
    if n == 0 || pos == 0 || pos > text.len_chars() {
        return None;
    }

    let mut count = 0;
    for (i, g) in text.graphemes_at(pos).reversed().enumerate() {
        if matcher.grapheme_match(g) {
            count = i + 1;
            n -= 1;
            if n == 0 {
                break;
            }
        }
    }

    (n == 0).then(|| nth_prev_grapheme_boundary(text, pos, count))
}

pub fn find_folded_nth_next(
    text: RopeSlice,
    annotations: &FoldAnnotations,
    matcher: impl GraphemeMatcher,
    pos: usize,
    mut n: usize,
) -> Option<usize> {
    if n == 0 || pos > text.len_chars() {
        return None;
    }

    let mut count = 0;
    for (i, g) in text
        .folded_graphemes_at(annotations, text.char_to_byte(pos))
        .skip(1)
        .enumerate()
    {
        if matcher.grapheme_match(g) {
            count = i + 1;
            n -= 1;
            if n == 0 {
                break;
            }
        }
    }

    (n == 0).then(|| nth_next_folded_grapheme_boundary(text, annotations, pos, count))
}

pub fn find_folded_nth_prev(
    text: RopeSlice,
    annotations: &FoldAnnotations,
    matcher: impl GraphemeMatcher,
    pos: usize,
    mut n: usize,
) -> Option<usize> {
    if n == 0 || pos == 0 || pos > text.len_chars() {
        return None;
    }

    let mut count = 0;
    for (i, g) in text
        .folded_graphemes_at(annotations, text.char_to_byte(pos))
        .reversed()
        .enumerate()
    {
        if matcher.grapheme_match(g) {
            count = i + 1;
            n -= 1;
            if n == 0 {
                break;
            }
        }
    }

    (n == 0).then(|| nth_prev_folded_grapheme_boundary(text, annotations, pos, count))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::movement::Direction;

    #[test]
    fn test_find_nth_char() {
        let text = RopeSlice::from("aa ⌚aa \r\n aa");

        // Forward direction
        assert_eq!(find_nth_char(1, text, 'a', 5, Direction::Forward), Some(5));
        assert_eq!(find_nth_char(2, text, 'a', 5, Direction::Forward), Some(10));
        assert_eq!(find_nth_char(3, text, 'a', 5, Direction::Forward), Some(11));
        assert_eq!(find_nth_char(4, text, 'a', 5, Direction::Forward), None);

        // Backward direction
        assert_eq!(find_nth_char(1, text, 'a', 5, Direction::Backward), Some(4));
        assert_eq!(find_nth_char(2, text, 'a', 5, Direction::Backward), Some(1));
        assert_eq!(find_nth_char(3, text, 'a', 5, Direction::Backward), Some(0));
        assert_eq!(find_nth_char(4, text, 'a', 5, Direction::Backward), None);

        // Edge cases
        assert_eq!(find_nth_char(0, text, 'a', 5, Direction::Forward), None); // n = 0
        assert_eq!(find_nth_char(1, text, 'x', 5, Direction::Forward), None); // Not found
        assert_eq!(find_nth_char(1, text, 'a', 20, Direction::Forward), None); // Beyond text
        assert_eq!(find_nth_char(1, text, 'a', 0, Direction::Backward), None); // At start going backward
    }
}
