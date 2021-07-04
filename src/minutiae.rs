/// This struct preserves only the last
/// 3 bits of a `u8`: 0x----_-aaa
#[derive(Clone, Copy, Debug)]
struct U3(u8);

impl From<u8> for U3 {
    fn from(byte: u8) -> Self {
        U3(byte & 0b111)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Style {
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
pub(super) enum Direction {
    Right,
    Left
}

impl From<bool> for Direction {
    fn from(b: bool) -> Self {
        if b { Direction::Right } else { Direction::Left }
    }

}

pub(super) struct DSPair(pub [(Direction, Style) ; 2]);

impl From<u8> for DSPair {
    fn from(byte: u8) -> Self {
        DSPair([
            (
                Direction::from(byte & 0b1 == 0), // 0bxxxx_xxxY
                Style::from(U3::from((byte >> 1) & 0b111)), // 0bxxxx_YYYx
                ),
            (
                Direction::from(((byte >> 4) & 0b1) == 0), // 0bxxxY_xxxx
                Style::from(U3::from(byte >> 5)), // 0bYYYx_xxxx
                )
        ])
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Note {
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
            Note::Full(Direction::Right, Style::Style6) => '+',
            Note::Full(Direction::Right, Style::Style7) => 'U',

            Note::Full(Direction::Left, Style::Style0) => '<',
            Note::Full(Direction::Left, Style::Style1) => '~',
            Note::Full(Direction::Left, Style::Style2) => '=',
            Note::Full(Direction::Left, Style::Style3) => '_',
            Note::Full(Direction::Left, Style::Style4) => '!',
            Note::Full(Direction::Left, Style::Style5) => '(',
            Note::Full(Direction::Left, Style::Style6) => 'T',
            Note::Full(Direction::Left, Style::Style7) => 'A',
        };
        write!(f, "{}", c)
    }
}

