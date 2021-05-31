#[derive(Clone, Copy, Debug)]
struct U3(u8);

impl From<u8> for U3 {
    fn from(byte: u8) -> Self {
        U3(byte & 0b111)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Style {
    Style0, Style1, Style2, Style3,
    Style4, Style5, Style6, Style7,
}

impl From<U3> for Style {
    fn from(byte: U3) -> Self {
        use Style::*;
        match byte.0 { 
            0b000 => Style0,
            0b001 => Style1,
            0b010 => Style2,
            0b011 => Style3,
            0b100 => Style4,
            0b101 => Style5,
            0b110 => Style6,
            0b111 => Style7,
            _ => unreachable!()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Direction {
    Right,
    Left
}

impl From<bool> for Direction {
    fn from(b: bool) -> Self {
        if b { Direction::Right } else { Direction::Left }
    }

}

struct DSPair([(Direction, Style) ; 2]);

impl From<u8> for DSPair {
    fn from(byte: u8) -> Self {
        DSPair([
            (
                Direction::from(byte & 0b1 == 0), // 0bxxxx_xxxY
                Style::from(U3::from((byte >> 1) & 0b111)), // 0xxxx_YYYx
                ),
            (
                Direction::from(((byte >> 4) & 0b1) == 0), // 0bxxxY_xxxx
                Style::from(U3::from(byte >> 5)), // 0bYYYx_xxxx
                )
        ])
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Note {
    Empty,
    Full(Direction, Style)
}

impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Note::Empty => ' ',

            Note::Full(Direction::Right, Style::Style0) => '>',
            Note::Full(Direction::Right, Style::Style1) => '.',
            Note::Full(Direction::Right, Style::Style2) => '*',
            Note::Full(Direction::Right, Style::Style3) => 'o',
            Note::Full(Direction::Right, Style::Style4) => 'x',
            Note::Full(Direction::Right, Style::Style5) => ')',
            Note::Full(Direction::Right, Style::Style6) => 'p',
            Note::Full(Direction::Right, Style::Style7) => 'b',

            Note::Full(Direction::Left, Style::Style0) => '<',
            Note::Full(Direction::Left, Style::Style1) => '~',
            Note::Full(Direction::Left, Style::Style2) => '=',
            Note::Full(Direction::Left, Style::Style3) => '_',
            Note::Full(Direction::Left, Style::Style4) => '!',
            Note::Full(Direction::Left, Style::Style5) => '(',
            Note::Full(Direction::Left, Style::Style6) => 'q',
            Note::Full(Direction::Left, Style::Style7) => 'd',
        };
        write!(f, "{}", c)
    }
}

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

// ---


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


