use crate::matching::{
    error::Error,
    state::{HammingDistance, Hashes, Match, Matches},
};

// Matches hashes and outputs the result
pub trait MatchProcessor: Send + Sync {
    type Input;
    type Output;
    type Error;
    fn process(&self, inputs: Self::Input) -> Result<Self::Output, Self::Error>;
}

/// Matches only unique pairs. (A,A) and (A,B) (B,A) are not allowed
#[derive(Default)]
pub struct UniquePairMatcher {}
impl MatchProcessor for UniquePairMatcher {
    type Error = Error;
    type Input = Hashes;
    type Output = Matches;
    fn process(&self, inputs: Self::Input) -> Result<Self::Output, Self::Error> {
        let mut matches: Matches = Matches::default();
        for (i, input1) in inputs.iter().enumerate() {
            for input2 in &inputs[i + 1..] {
                let hamming_distance = compute_hamming_distance(input1.hash(), input2.hash())?;
                let res = Match::new(input1.id(), input2.id(), hamming_distance);
                matches.push(res);
            }
        }
        Ok(matches)
    }
}

fn compute_hamming_distance(x: &[u8], y: &[u8]) -> Result<HammingDistance, Error> {
    if x.len() != y.len() {
        return Err(Error::HashesNotEqualLength {
            l1: x.len() as u32,
            l2: y.len() as u32,
        });
    }
    let hamming_distance = x.iter().zip(y).map(|(x, y)| (x ^ y).count_ones()).sum();
    return Ok(HammingDistance::new(hamming_distance, x.len() as u32));
}
