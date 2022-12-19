use parsink::{Inst, Pattern, Step, Weight};
use std::num::NonZeroU128;

#[derive(Clone, Debug, Eq, PartialEq)]
struct CaesarKeys(NonZeroU128);

impl Weight for CaesarKeys {
    fn success() -> Self {
        CaesarKeys(NonZeroU128::new(!0).unwrap())
    }

    fn concat(&self, other: &Self) -> Option<Self> {
        NonZeroU128::new(self.0.get() & other.0.get()).map(CaesarKeys)
    }

    fn merge(&mut self, other: Self) {
        self.0 |= other.0;
    }
}

struct CaesarRange(std::ops::RangeInclusive<u8>);

impl Step<u8, CaesarKeys> for CaesarRange {
    fn step(&self, &input: &u8) -> Option<CaesarKeys> {
        // If C=P+K (mod 128), then K=C-P (mod 128).
        let mask = (1u128 << self.0.len()) - 1;
        let key_start = input.wrapping_sub(*self.0.end()) % 128;
        let mask = mask.rotate_left(key_start.into());
        //println!("C={:?}, P={:?}-{:?}: {:0128b}", input as char, *self.0.start() as char, *self.0.end() as char, mask);
        NonZeroU128::new(mask).map(CaesarKeys)
    }
}

fn main() {
    let mut pattern = Pattern::new(&[
        // 0
        Inst::Step(CaesarRange(b'a'..=b'z')),
        // 1
        Inst::PreferTarget(0u8),
        // 2
        Inst::Step(CaesarRange(b'_'..=b'_')),
        // 3
        Inst::Step(CaesarRange(b'a'..=b'z')),
        // 4
        Inst::PreferTarget(3u8),
    ]);

    let message = b"hello_world";
    for key in 0..128 {
        let ciphertext = Vec::from_iter(message.iter().map(|&p| p.wrapping_add(key) & 0x7F));
        let expected = CaesarKeys(NonZeroU128::new(1u128 << key).unwrap());
        let result = pattern.eval(&ciphertext).unwrap();
        assert_eq!(result, expected, "no match for key {}\n{:128b}\n{:128b}", key, result.0, expected.0);
    }
}
