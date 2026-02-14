use std::{
    sync::mpsc::{Receiver, sync_channel},
    thread,
};

use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};

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

/// Matches only unique pairs, (A,A) and (A,B) (B,A) are not allowed. All results are gathered and
/// returned, potentially eating all memory.
#[derive(Default)]
pub struct UniquePairMatcher {}
impl MatchProcessor for UniquePairMatcher {
    type Error = Error;
    type Input = Hashes;
    type Output = Matches;
    fn process(&self, inputs: Self::Input) -> Result<Self::Output, Self::Error> {
        let style = ProgressStyle::with_template(
            "[{elapsed_precise} | {eta_precise}] Matching hashes: {pos:>7}/{len:7} {percent}%",
        )
        .unwrap()
        .progress_chars("##-");
        let mut matches: Matches = Matches::default();
        for (i, input1) in inputs.iter().enumerate().progress_with_style(style) {
            for input2 in inputs[i + 1..].iter() {
                let hamming_distance = compute_hamming_distance(input1.hash(), input2.hash())?;
                let res = Match::new(input1.id(), input2.id(), hamming_distance);
                matches.push(res);
            }
        }
        Ok(matches)
    }
}

#[derive(Debug, Default)]
pub struct ThreadedUniquePairMatcher {}
impl MatchProcessor for ThreadedUniquePairMatcher {
    type Error = Error;
    type Input = Hashes;
    type Output = Receiver<Match>;
    fn process(&self, inputs: Self::Input) -> Result<Self::Output, Self::Error> {
        let (tx, rx) = sync_channel(10_000);
        let style = ProgressStyle::with_template(
            "[{elapsed_precise} | {eta_precise}] Matching hashes: {pos:>7}/{len:7} {percent}%",
        )
        .unwrap()
        .progress_chars("##-");
        let n = inputs.len();
        let pb = ProgressBar::new((n * (n - 1) / 2) as u64);
        pb.set_message("starting matching");
        pb.set_style(style);
        thread::spawn(move || {
            let mut should_quit = false;
            for (i, input1) in inputs.iter().enumerate() {
                if should_quit {
                    break;
                }
                pb.set_message("matching...");
                for input2 in inputs[i + 1..].iter() {
                    pb.inc(1);
                    let res = compute_hamming_distance(input1.hash(), input2.hash());
                    let hamming_distance = match res {
                        Ok(r) => r,
                        Err(e) => {
                            tracing::warn!("could not match entry: {}", e);
                            should_quit = true;
                            break;
                        }
                    };

                    let res = Match::new(input1.id(), input2.id(), hamming_distance);
                    if let Err(e) = tx.send(res) {
                        tracing::warn!("could not send result to channel, err: {}", e);
                        break;
                    };
                }
            }
            pb.finish();
        });
        Ok(rx)
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
    Ok(HammingDistance::new(hamming_distance, x.len() as u32))
}
