// This file implements the main algorithm.

mod minutiae;
use minutiae::{Note, Direction, Style, DSPair};

/// The mental model of this library one should have
/// is that of a drunken diver descending into the depths of the ocean
/// while leaving behind a trail of notes 
/// each with a stylized depiction of a direction (left or right), 
/// allowing those that come afterwards to swim the same path the diver took.


/// A row of `Note`s.
/// If the diver is in this row, it contains
/// the current position of the diver.
/// Otherwise, it contains the diver's position when it left the row.
///
// `usize` should be within `0` and `WIDTH`.
// NB: We consider `0` to be the leftmost position and `WIDTH` to be the rightmost position.
#[derive(PartialEq)]
pub struct Row<const WIDTH: usize>([Note ; WIDTH], usize);

impl <const WIDTH: usize> Row<WIDTH> {

    const EMPTY_ARR: [ Note ; WIDTH ] = [ Note::Empty ; WIDTH ];
    
    // NB: The goal of `WIDTH/8` is to increase the margin size
    // e.g. with `0..0`, we get
    // |<
    // |<
    // |<
    // etc.
    //
    // while with `WIDTH/8` we increase the minimum of the sum 
    // of the margins of both sides in proportion to the width
    // of the route. Note that `8` is a magic number,
    // but it is meant to strike a balance between `4`
    // which seems to allow notes to easily encompass the center of
    // the route, and `16` which seems a little too small.
    const MIN_MARGIN: usize = WIDTH/8;

    fn new(loc: usize) -> Row<WIDTH> { Row(Row::<WIDTH>::EMPTY_ARR, loc) }

    fn set_note(&mut self, note: Note) { self.0[self.1] = note; }
    fn is_note_empty(&self) -> bool { self.0[self.1] == Note::Empty }

    // Get the location of the diver or where it left the row.
    fn get_loc(&self) -> &usize { &self.1 }

    fn is_empty(&self) -> bool { self.0 == Row::<WIDTH>::EMPTY_ARR }

    fn is_rightmost(&self) -> bool { self.1 + 1 >= WIDTH } 
    fn is_leftmost(&self) -> bool { self.1 <= 0 }

    fn go_rightmost(&mut self) { self.1 = WIDTH.saturating_sub(1) }
    fn go_leftmost(&mut self) { self.1 = 0 }

    fn go_right_unchecked(&mut self) { self.1 += 1 }
    fn go_left_unchecked(&mut self) { self.1 -= 1 }

    fn go_right_wrapping(&mut self) { 
        if self.is_rightmost() { self.go_leftmost() } else { self.go_right_unchecked() } 
    }
    fn go_left_wrapping(&mut self) {
        if self.is_leftmost() { self.go_rightmost() } else { self.go_left_unchecked() }
    }

    /// Moves the diver rightward.
    /// Returns whether the diver should go down to a new row.
    /// true - yes, the diver should go down
    /// false - no, the diver shouldn't go down
    fn journey_right(&mut self) -> bool {
        // examine the current location. Is it empty? Then settle there.
        // Otherwise, go to the right, wrapping.
        for _ in 0..Row::<WIDTH>::MIN_MARGIN {
            if self.is_note_empty() { return false }
            else { self.go_right_wrapping() }
        }
        // examine the current location. Is it empty? Then settle there.
        // Is it at the end? Then go down a row from before you started the journey.
        // Otherwise, go right.
        loop {
            if self.is_note_empty() { return false }
            else if self.is_rightmost() { return true } // end of the line
            else { self.go_right_unchecked(); }
        }
    }

    /// See `journey_right`.
    fn journey_left(&mut self) -> bool {
        for _ in 0..Row::<WIDTH>::MIN_MARGIN {
            if self.is_note_empty() { return false }
            else { self.go_left_wrapping() }
        }
        loop {
            if self.is_note_empty() { return false }
            else if self.is_leftmost() { return true } // end of the line
            else { self.go_left_unchecked(); }
        }
    }
}


impl <const WIDTH: usize> std::fmt::Display for Row<WIDTH> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for note in self.0.iter() { write!(f, "{}", note)? }
        Ok(())
    }
}


/// An iterator over the rows a diver descends through.
pub struct Dive<T, const WIDTH: usize> {
    iter: T,
    row: Row<WIDTH>,
    buffered: Option<(Direction, Style)>,
}

impl <T: Iterator<Item=u8>, const WIDTH: usize> From<T> for Dive<T, WIDTH> {
    fn from(it: T) -> Dive<T, WIDTH> {
        Dive {
            iter: it,
            row: Row::new(WIDTH/2),
            buffered: None,
        }
    }
}

impl <T: Iterator<Item=u8>, const WIDTH: usize> Dive<T, WIDTH> {

    /// Makes the diver move towards a direction, eventually settling.
    ///
    /// Returns None if the diver stayed on the same row.
    /// Returns the old row if the diver left the row.
    fn settle(&mut self, d: Direction, home: usize) -> Option<Row<WIDTH>> {
        let go_down = match d {
            Direction::Right => { self.row.journey_right() },
            Direction::Left => { self.row.journey_left() }
        };

        if go_down {
           return Some(std::mem::replace(&mut self.row, Row::new(home)))
        }
        else { None }
    }

    /// When the diver decides to `go`, it first leaves behind
    /// a `Note` at that location detailing the `Direction` `d` it went.
    /// The `Note` has a `Style`.
    ///
    /// Let `home` be the starting position of the diver.
    /// Repeat MIN_MARGIN times:
    ///     The diver goes one space in direction `d`,
    ///     wrapping to the other side of the row if necessary.
    ///     Then, if the space is empty, return `None`,
    ///     as the diver did not go down a row.
    /// Repeat:
    ///     If going one space in direction `d` would put the diver
    ///     past the edges of the row,
    ///     create a new row with the diver starting at `home`,
    ///     then return the old row.
    ///     Otherwise, the diver goes one space in direction `d`.
    ///     If the space is empty, return `None`.
    ///     
    fn go(&mut self, d: Direction, s: Style) -> Option<Row<WIDTH>> {
        self.row.set_note(Note::Full(d, s));
        self.settle(d, *self.row.get_loc())
    }
}

impl <T: Iterator<Item=u8>, const WIDTH: usize> Iterator for Dive<T, WIDTH> {
    type Item = Row<WIDTH>;

    /// `go` on the buffer if a value is present.
    /// If the diver descends, return the obtained row.
    /// 
    /// Repeat:
    ///     Retrieve the next value in `iter`.
    ///     Process the `u8`  into two `u4`'s, which convert to
    ///     two tuples of `Direction` and `Style`.
    ///     `go` on the first tuple. If the diver descends, buffer the second tuple
    ///     and return the obtained row.
    ///     Otherwise, `go`on the second tuple. If the diver descends,
    ///     return the obtained row.
    fn next(&mut self) -> Option<Self::Item> {
        macro_rules! go_and_return_on_descent {
            ($direction:ident, $style:ident $(,  $b:block)?) => {
                if let Some(row) = self.go($direction, $style) {
                    $($b)?
                    return Some(row)
                }
            }
        }

        if let Some((d, s)) = self.buffered.take() {
            go_and_return_on_descent!(d, s)
        }

        while let Some(byte) = self.iter.next() {
                let DSPair([(d1, s1), (d2, s2)]) = DSPair::from(byte);
                go_and_return_on_descent!(d1, s1, { self.buffered = Some((d2, s2)); });
                go_and_return_on_descent!(d2, s2)
        }
        if self.row.is_empty() { return None }
        else {
            let orig = *self.row.get_loc();
            return Some(std::mem::replace(&mut self.row, Row::new(orig)))
        }
    }
}


/// A completed route the diver took.
///
/// This is the main way to print the art the diver generates.
pub struct Route<const WIDTH: usize> {
    route: Vec<Row<WIDTH>>,
    end: usize,
}


impl <T: Iterator<Item=u8>, const WIDTH: usize> From<Dive<T, WIDTH>> for Route<WIDTH> {
    fn from(it: Dive<T, WIDTH>) -> Route<WIDTH> {
        let vec: Vec<_> = it.collect(); 
        Route{ end: vec.last().map(|Row(_, v)| *v).unwrap_or(WIDTH/2), route: vec }
    }

}

impl <const WIDTH: usize> std::fmt::Display for Route<WIDTH> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "+{}v{}+\n", "-".repeat((WIDTH/2).saturating_sub(1)), "-".repeat(WIDTH - (WIDTH/2)))?;
        for row in &self.route {
            write!(f, "|{}|\n", row)?
        }
        write!(f, "+{}v{}+\n", "-".repeat(self.end), "-".repeat((WIDTH - self.end).saturating_sub(1)))
    }
}


