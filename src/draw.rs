use std::collections::BTreeSet;
use std::fs;

use anyhow::{bail, Context, Result};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::model::DrawResult;

pub fn pick_winners(path: &str, count: usize, seed: u64) -> Result<DrawResult> {
    let content = fs::read_to_string(path).with_context(|| format!("failed to read {path}"))?;
    let mut candidates = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        bail!("input file does not contain any candidates");
    }

    let count = count.min(candidates.len());
    let mut rng = StdRng::seed_from_u64(seed);
    candidates.shuffle(&mut rng);

    Ok(DrawResult {
        winners: candidates.into_iter().take(count).collect(),
        total_candidates: content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count(),
    })
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use tempfile::NamedTempFile;

    use super::pick_winners;

    #[test]
    fn draw_deduplicates_entries() -> Result<()> {
        let file = NamedTempFile::new()?;
        std::fs::write(file.path(), "alice\nbob\nalice\n")?;

        let result = pick_winners(file.path().to_str().unwrap_or_default(), 2, 7)?;

        assert_eq!(result.total_candidates, 3);
        assert_eq!(result.winners.len(), 2);
        Ok(())
    }
}
