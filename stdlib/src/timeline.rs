use anyhow::{bail, Result};
use serde_json::{json, Value};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

pub fn build(value: &Value) -> Result<Value> {
    let rows = match value {
        Value::Array(rows) => rows.as_slice(),
        Value::Object(map) => ["events", "processes", "connections", "modules"]
            .iter().find_map(|key| map.get(*key).and_then(Value::as_array))
            .map(Vec::as_slice).ok_or_else(|| anyhow::anyhow!("timeline.build expects dated records"))?,
        _ => bail!("timeline.build expects dated records"),
    };
    let mut events = Vec::new();
    let mut omitted_undated = 0;
    for row in rows {
        let Some(timestamp) = row.get("timestamp").and_then(Value::as_str) else {
            omitted_undated += 1;
            continue;
        };
        let Ok(parsed) = OffsetDateTime::parse(timestamp, &Rfc3339) else {
            omitted_undated += 1;
            continue;
        };
        events.push((parsed, row.clone()));
    }
    events.sort_by_key(|(timestamp, _)| *timestamp);
    Ok(json!({"events": events.into_iter().map(|(_, row)| row).collect::<Vec<_>>(), "omitted_undated": omitted_undated}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sorts_instants_not_offset_strings() {
        let input = json!([
            {"timestamp":"2026-01-01T01:00:00+01:00","id":1},
            {"timestamp":"2025-12-31T23:30:00Z","id":2},
            {"id":3}
        ]);
        let result = build(&input).unwrap();
        assert_eq!(result["events"][0]["id"], 2);
        assert_eq!(result["omitted_undated"], 1);
    }

    #[test]
    fn rejects_non_event_objects() {
        assert!(build(&json!({"digest": "abc"})).is_err());
    }
}
