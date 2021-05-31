
mod minutiae;
use minutiae::{Note, Direction, Style, DSPair};


#[derive(PartialEq)]
pub struct Row<const WIDTH: usize>([Note ; WIDTH], usize);

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
            row: Row([ Note::Empty ; WIDTH ], WIDTH/2),
            buffered: None,
        }
    }
}

impl <T: Iterator<Item=u8>, const WIDTH: usize> Dive<T, WIDTH> {
    fn at_end(&self) -> bool { self.row.1 + 1 >= WIDTH } 
    fn at_beginning(&self) -> bool { self.row.1 == 0 }

    fn settle(&mut self, d: Direction, home: usize) -> Option<Row<WIDTH>> {
        // move one over, wrapping
        // so instead of looking at the next spot, we examine the current spot.
        match d {
            Direction::Right => if self.at_end() { self.row.1 = 0 } else { self.row.1 += 1 },
            Direction::Left => 
                if self.at_beginning() { self.row.1 = WIDTH.saturating_sub(1) } else { self.row.1 -= 1 }
        }

        // examine current location: is it empty? then settle there. Is it at the end? go down. Otherwise,
        // go in direction `d` and repeat.
        let go_down = match d {
            Direction::Right => {
                loop {
                    if let Note::Empty = self.row.0[self.row.1] { break false }
                    else if self.at_end() { break true } // end of the line
                    else { self.row.1 += 1; }
                }
            },
            Direction::Left => {
                loop {
                    if let Note::Empty = self.row.0[self.row.1] { break false }
                    else if self.at_beginning() { break true } // end of the line
                    else { self.row.1 -= 1; }
                }
            }
        };

        if go_down {
           let tmp = std::mem::replace(&mut self.row, Row([Note::Empty; WIDTH], home));
           return Some(tmp)
        }
        else { None }
    }

    fn go(&mut self, d: Direction, s: Style) -> Option<Row<WIDTH>> {
        self.row.0[self.row.1] = Note::Full(d, s);
        self.settle(d, self.row.1)
    }

}

impl <T: Iterator<Item=u8>, const WIDTH: usize> Iterator for Dive<T, WIDTH> {
    type Item = Row<WIDTH>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((d, s)) = self.buffered.take() {
            if let Some(row) = self.go(d, s) {
                return Some(row)
            }
        }

        loop {
            if let Some(byte) = self.iter.next() {
                let DSPair([(d1, s1), (d2, s2)]) = DSPair::from(byte);
                if let Some(row) = self.go(d1, s1) {
                    self.buffered = Some((d2, s2));
                    break Some(row)
                }
                if let Some(row) = self.go(d2, s2) {
                    break Some(row)
                }
            }
            else {
                if [Note::Empty; WIDTH] == self.row.0 {
                    break None
                }
                else {
                    let orig = self.row.1;
                    let tmp = std::mem::replace(&mut self.row, Row([Note::Empty; WIDTH], orig));
                    break Some(tmp)
                }
            }
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


