
mod minutiae;
use minutiae::{Note, Direction, Style, DSPair};


/// The `usize` dicatates either where the diver currently is,
/// or where the diver ended up before deciding to go down
///
/// Note: `usize` should be within `0` and `WIDTH`
#[derive(PartialEq)]
pub struct Row<const WIDTH: usize>([Note ; WIDTH], usize);

impl <const WIDTH: usize> Row<WIDTH> {
    const EMPTY_ARR: [ Note ; WIDTH ] = [ Note::Empty ; WIDTH ];

    fn new(loc: usize) -> Row<WIDTH> { Row(Row::<WIDTH>::EMPTY_ARR, loc) }

    fn set_note(&mut self, note: Note) { self.0[self.1] = note; }
    fn is_note_empty(&self) -> bool { self.0[self.1] == Note::Empty }

    fn get_loc(&self) -> &usize { &self.1 }

    fn is_empty(&self) -> bool { self.0 == Row::<WIDTH>::EMPTY_ARR }
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
    fn is_rightmost(&self) -> bool { self.row.1 + 1 >= WIDTH } 
    fn is_leftmost(&self) -> bool { self.row.1 == 0 }

    fn go_rightmost(&mut self) { self.row.1 = WIDTH.saturating_sub(1) }
    fn go_leftmost(&mut self) { self.row.1 = 0 }

    fn go_right_unchecked(&mut self) { self.row.1 += 1 }
    fn go_left_unchecked(&mut self) { self.row.1 -= 1 }

    fn go_right_wrapping(&mut self) { 
        if self.is_rightmost() { self.go_leftmost() } else { self.go_right_unchecked() } 
    }
    fn go_left_wrapping(&mut self) {
        if self.is_leftmost() { self.go_rightmost() } else { self.go_left_unchecked() }
    }

    fn settle(&mut self, d: Direction, home: usize) -> Option<Row<WIDTH>> {
        // move one over, wrapping
        // so instead of looking at the next spot, we examine the current spot.
        match d {
            Direction::Right => self.go_right_wrapping(), 
            Direction::Left => self.go_left_wrapping(),
        }

        // examine current location: is it empty? then settle there. Is it at the end? go down. Otherwise,
        // go in direction `d` and repeat.
        let go_down = match d {
            Direction::Right => {
                loop {
                    if self.row.is_note_empty() { break false }
                    else if self.is_rightmost() { break true } // end of the line
                    else { self.go_right_unchecked(); }
                }
            },
            Direction::Left => {
                loop {
                    if self.row.is_note_empty() { break false }
                    else if self.is_leftmost() { break true } // end of the line
                    else { self.go_left_unchecked(); }
                }
            }
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
        macro_rules! go_return {
            ($direction:ident, $style:ident $(,  $b:block)?) => {
                if let Some(row) = self.go($direction, $style) {
                    $($b)?
                    return Some(row)
                }
            }
        }


        if let Some((d, s)) = self.buffered.take() {
            go_return!(d, s)
        }

        while let Some(byte) = self.iter.next() {
                let DSPair([(d1, s1), (d2, s2)]) = DSPair::from(byte);
                go_return!(d1, s1, 
                           {self.buffered = Some((d2, s2));} 
                           );
                go_return!(d2, s2)
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


