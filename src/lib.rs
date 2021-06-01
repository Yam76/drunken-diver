
mod minutiae;
use minutiae::{Note, Direction, Style, DSPair};


/// A row of `Note`s, with either the current position of the diver
/// or the diver's position when it left the row.
///
// `usize` should be within `0` and `WIDTH`.
// We consider `0` to the be the leftmost, and `WIDTH` to the be the rightmost.
#[derive(PartialEq)]
pub struct Row<const WIDTH: usize>([Note ; WIDTH], usize);

impl <const WIDTH: usize> Row<WIDTH> {
    // NB: We consider `0` to be the leftmost position and `WIDTH` to be the rightmost position.

    const EMPTY_ARR: [ Note ; WIDTH ] = [ Note::Empty ; WIDTH ];

    fn new(loc: usize) -> Row<WIDTH> { Row(Row::<WIDTH>::EMPTY_ARR, loc) }

    fn set_note(&mut self, note: Note) { self.0[self.1] = note; }
    fn is_note_empty(&self) -> bool { self.0[self.1] == Note::Empty }

    // Get the location of the diver or where it left the row.
    fn get_loc(&self) -> &usize { &self.1 }

    fn is_empty(&self) -> bool { self.0 == Row::<WIDTH>::EMPTY_ARR }

    fn is_rightmost(&self) -> bool { self.1 + 1 >= WIDTH } 
    fn is_leftmost(&self) -> bool { self.1 == 0 }

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
    fn journey_right(&mut self) -> bool {
        // examine the current location. Is it empty? Then settle there.
        // Otherwise, go to the right, wrapping.
        for _ in 0..(WIDTH/8) {
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

    fn journey_left(&mut self) -> bool {
        for _ in 0..(WIDTH/8) {
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
        for note in self.0.iter() {
            write!(f, "{}", note)?;
        }
        Ok(())
    }
}



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

    fn settle(&mut self, d: Direction, home: usize) -> Option<Row<WIDTH>> {
        let go_down = match d {
            Direction::Right => { self.row.journey_right() },
            Direction::Left => { self.row.journey_left() }
        };

        if go_down {
           let tmp = std::mem::replace(&mut self.row, Row::new(home));
           return Some(tmp)
        }
        else { None }
    }

    fn go(&mut self, d: Direction, s: Style) -> Option<Row<WIDTH>> {
        self.row.set_note(Note::Full(d, s));
        self.settle(d, *self.row.get_loc())
    }
}

impl <T: Iterator<Item=u8>, const WIDTH: usize> Iterator for Dive<T, WIDTH> {
    type Item = Row<WIDTH>;

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
            let tmp = std::mem::replace(&mut self.row, Row::new(orig));
            return Some(tmp)
        }
    }
}



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


