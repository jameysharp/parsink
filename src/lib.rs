use std::borrow::Borrow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::marker::PhantomData;

/// For types which represent information extracted from a match.
///
/// This is analogous to the concept of a "weighted semiring" from discrete
/// mathematics. `success` should return "one", `concat` is "multiply", and
/// `merge` is "add". This library represents "zero" using [Option::None], and
/// the `concat` and `merge` methods of this trait are only used for combining
/// non-zero weights.
pub trait Weight: Sized {
    /// The initial weight to use before matching begins.
    fn success() -> Self;
    /// How to combine weights when `self` is the result of one successful match
    /// that is followed by another successful match returning `other`. Even
    /// though both matches individually succeeded, they might not be valid
    /// together, so their concatenation may fail.
    fn concat(&self, other: &Self) -> Option<Self>;
    /// How to combine weights when `self` is the successful result of a
    /// higher-priority alternative and `other` is the successful result of a
    /// lower-priority alternative on the same input. When both alternatives
    /// succeed, there is definitely a valid match, so their merge is not
    /// allowed to fail. It may ignore either match result, however, if the
    /// static priorities are sufficient.
    fn merge(&mut self, other: Self);
}

/// A trivial weight for determining whether the input matched the language.
impl Weight for () {
    fn success() -> Self {
        ()
    }

    fn concat(&self, _: &Self) -> Option<Self> {
        Some(())
    }

    fn merge(&mut self, _: Self) {}
}

/// For types which can describe whether a single input matches.
pub trait Step<T, W> {
    fn step(&self, input: &T) -> Option<W>;
}

impl<T: PartialOrd, W: Weight> Step<T, W> for std::ops::RangeInclusive<T> {
    /// Returns [Weight::success] if the input is within this range.
    fn step(&self, input: &T) -> Option<W> {
        if self.contains(input) {
            Some(W::success())
        } else {
            None
        }
    }
}

impl<T, W, F: Fn(&T) -> Option<W>> Step<T, W> for F {
    /// Returns the weight computed by the given function or closure.
    fn step(&self, input: &T) -> Option<W> {
        self(input)
    }
}

pub enum Inst<S, PC = u16> {
    /// Evaluate the given [Step] on the current input. Discard the thread if
    /// either `step` or `concat` reports that the match has failed. Otherwise,
    /// update the current thread's weight and continue executing at the
    /// instruction after this one.
    Step(S),
    /// Continue executing this thread at the given program counter.
    Jump(PC),
    /// Continue executing this thread at the given program counter, and add a
    /// lower-priority thread that starts at the instruction after this one.
    PreferTarget(PC),
    /// Continue executing this thread at the instruction after this one, and
    /// add a lower-priority thread that starts at the given program counter.
    PreferNext(PC),
}

pub struct Pattern<'a, S, PC, T, W> {
    pattern: &'a [Inst<S, PC>],
    threads: Vec<(PC, W)>,
    index: HashMap<PC, usize>,
    _phantom: PhantomData<T>,
}

impl<'a, S, PC, T, W> Pattern<'a, S, PC, T, W>
where
    S: Step<T, W>,
    W: Weight + Clone,
    PC: Into<usize> + Copy + Eq + std::hash::Hash,
    usize: TryInto<PC>,
{
    pub fn new(pattern: &'a [Inst<S, PC>]) -> Self {
        Pattern {
            pattern,
            threads: Vec::new(),
            index: HashMap::new(),
            _phantom: PhantomData,
        }
    }

    pub fn eval<B: Borrow<T>, I: IntoIterator<Item = B>>(&mut self, input: I) -> Option<W> {
        self.threads.clear();
        self.threads.push((Self::as_pc(0), W::success()));
        let mut result = None;
        for input in input {
            let input = input.borrow();
            let mut matched = None;
            self.index.clear();
            for (pc, weight) in std::mem::take(&mut self.threads) {
                matched = merge(matched, self.add(pc.into(), &weight, input));
            }
            result = matched.or(result);
            if self.threads.is_empty() {
                break;
            }
        }
        result
    }

    fn add(&mut self, pc: usize, weight: &W, input: &T) -> Option<W> {
        match self.pattern.get(pc) {
            // Walking off the end of the pattern indicates a successful match.
            None => Some(weight.clone()),

            Some(Inst::Step(s)) => {
                if let Some(new) = s.step(input).and_then(|cur| weight.concat(&cur)) {
                    let next = Self::as_pc(pc + 1);
                    match self.index.entry(next) {
                        Entry::Vacant(entry) => {
                            entry.insert(self.threads.len());
                            self.threads.push((next, new));
                        }
                        Entry::Occupied(entry) => {
                            let (_pc, old) = &mut self.threads[*entry.get()];
                            old.merge(new);
                        }
                    }
                }
                None
            }
            Some(&Inst::Jump(to)) => self.add(to.into(), weight, input),
            Some(&Inst::PreferTarget(to)) => merge(
                self.add(to.into(), weight, input),
                self.add(pc + 1, weight, input),
            ),
            Some(&Inst::PreferNext(to)) => merge(
                self.add(pc + 1, weight, input),
                self.add(to.into(), weight, input),
            ),
        }
    }

    fn as_pc(pc: usize) -> PC {
        pc.try_into()
            .unwrap_or_else(|_| panic!("PC {} out of range", pc))
    }
}

fn merge<W: Weight>(a: Option<W>, b: Option<W>) -> Option<W> {
    match (a, b) {
        (None, None) => None,
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (Some(mut a), Some(b)) => {
            a.merge(b);
            Some(a)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognize() {
        let mut pattern = Pattern::new(&[
            Inst::Step(b'A'..=b'A'),
            Inst::Step(b'a'..=b'z'),
            Inst::PreferTarget(0u8),
        ]);

        assert_eq!(pattern.eval(b"0"), None);
        assert_eq!(pattern.eval(b"AbAz0"), Some(()));
    }
}
