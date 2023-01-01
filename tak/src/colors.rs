use takparse::Color;

#[derive(Clone, Copy, Debug, Hash, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Colors {
    bits: u64,
    len: u8,
}

impl Colors {
    pub const fn of_one(color: Color) -> Self {
        Self {
            bits: from_color(color),
            len: 1,
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn len(&self) -> usize {
        self.len as usize
    }

    pub const fn top(&self) -> Option<Color> {
        if self.is_empty() {
            return None;
        }
        Some(to_color(self.bits & 1))
    }

    pub fn push(&mut self, color: Color) {
        assert!(self.len < 64);
        self.bits = (self.bits << 1) | u64::from(color == Color::White);
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<Color> {
        if self.is_empty() {
            return None;
        }
        let color = to_color(self.bits & 1);
        self.bits >>= 1;
        self.len -= 1;
        Some(color)
    }

    pub fn take(&mut self, amount: u8) -> Self {
        assert!(amount <= self.len);
        let mask: u64 = !(!0 << amount);
        let taken = Self {
            bits: self.bits & mask,
            len: amount,
        };
        self.bits >>= amount;
        self.len -= amount;
        taken
    }
}

impl IntoIterator for Colors {
    type IntoIter = ColorsIter;
    type Item = Color;

    fn into_iter(self) -> Self::IntoIter {
        ColorsIter(Self {
            bits: self.bits.reverse_bits() >> (u64::BITS - u32::from(self.len)),
            len: self.len,
        })
    }
}

pub struct ColorsIter(Colors);

impl Iterator for ColorsIter {
    type Item = Color;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.0.len as usize;
        (len, Some(len))
    }
}

const fn to_color(n: u64) -> Color {
    if n == 0 {
        Color::Black
    } else {
        Color::White
    }
}

const fn from_color(color: Color) -> u64 {
    match color {
        Color::White => 1,
        Color::Black => 0,
    }
}

impl FromIterator<Color> for Colors {
    fn from_iter<T: IntoIterator<Item = Color>>(iter: T) -> Self {
        let mut colors = Self::default();
        for color in iter {
            colors.push(color);
        }
        colors
    }
}

#[cfg(test)]
mod tests {
    use takparse::Color;

    use super::{from_color, to_color, Colors};

    #[test]
    fn color_num() {
        assert_eq!(Color::White, to_color(from_color(Color::White)));
        assert_eq!(Color::Black, to_color(from_color(Color::Black)));
        assert_eq!(from_color(Color::White), 1);
        assert_eq!(from_color(Color::Black), 0);
    }

    #[test]
    fn push_pop() {
        let mut colors = Colors::default();
        colors.push(Color::White);
        colors.push(Color::White);
        colors.push(Color::Black);
        colors.push(Color::White);
        colors.push(Color::Black);

        assert_eq!(colors.pop(), Some(Color::Black));
        assert_eq!(colors.pop(), Some(Color::White));
        assert_eq!(colors.pop(), Some(Color::Black));
        assert_eq!(colors.pop(), Some(Color::White));
        assert_eq!(colors.pop(), Some(Color::White));
        assert_eq!(colors.pop(), None);
    }

    #[test]
    fn iter() {
        let mut colors = Colors::of_one(Color::White);
        colors.push(Color::Black);
        colors.push(Color::Black);
        colors.push(Color::White);
        colors.push(Color::White);
        colors.push(Color::Black);

        let v: Vec<_> = colors.into_iter().collect();
        assert_eq!(v, vec![
            Color::White,
            Color::Black,
            Color::Black,
            Color::White,
            Color::White,
            Color::Black
        ]);
    }

    #[test]
    fn take() {
        let mut colors = Colors::of_one(Color::White);
        colors.push(Color::Black);
        colors.push(Color::White);
        colors.push(Color::Black);
        colors.push(Color::Black);
        colors.push(Color::White);

        let mut a = colors.take(5);
        assert_eq!(a.pop(), Some(Color::White));
        assert_eq!(a.pop(), Some(Color::Black));
        assert_eq!(a.pop(), Some(Color::Black));
        assert_eq!(a.pop(), Some(Color::White));
        assert_eq!(a.pop(), Some(Color::Black));
        assert_eq!(a.pop(), None);

        assert_eq!(colors.pop(), Some(Color::White));
        assert_eq!(colors.pop(), None);
    }
}
