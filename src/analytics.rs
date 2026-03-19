use std::collections::BTreeMap;

use crate::model::{AnalyticsSummary, Event, KindStat};

pub fn summarize(events: &[Event]) -> AnalyticsSummary {
    let total_events = events.len();
    let total_score = events.iter().map(|event| event.score).sum::<u32>();
    let average_score = if total_events == 0 {
        0.0
    } else {
        total_score as f64 / total_events as f64
    };
    let highest_score = events.iter().map(|event| event.score).max().unwrap_or(0);

    let mut grouped = BTreeMap::new();
    for event in events {
        let entry = grouped
            .entry(event.kind.to_string())
            .or_insert_with(|| KindStat {
                kind: event.kind.clone(),
                count: 0,
                total_score: 0,
            });
        entry.count += 1;
        entry.total_score += event.score;
    }

    let mut top_events = events.to_vec();
    top_events.sort_by(|left, right| right.score.cmp(&left.score));
    top_events.truncate(5);

    AnalyticsSummary {
        total_events,
        total_score,
        average_score,
        highest_score,
        by_kind: grouped.into_values().collect(),
        top_events,
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::model::{Event, EventKind};

    use super::summarize;

    #[test]
    fn computes_summary_metrics() {
        let events = vec![
            Event {
                id: "1".to_string(),
                handle: "@x".to_string(),
                kind: EventKind::Tweet,
                message: "a".to_string(),
                score: 10,
                timestamp: Utc::now(),
            },
            Event {
                id: "2".to_string(),
                handle: "@x".to_string(),
                kind: EventKind::Reply,
                message: "b".to_string(),
                score: 20,
                timestamp: Utc::now(),
            },
        ];

        let summary = summarize(&events);

        assert_eq!(summary.total_events, 2);
        assert_eq!(summary.total_score, 30);
        assert_eq!(summary.highest_score, 20);
    }
}
