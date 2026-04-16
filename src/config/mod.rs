use serde_json::{json, Value};
use std::collections::BTreeMap;

pub type Config = BTreeMap<String, Value>;

pub fn default() -> Config {
    let mut c = BTreeMap::new();

    c.insert("Tab Size".to_string(), json!(4));

    c.insert("Auto Save".to_string(), json!(false));
    c.insert("Time To Auto Save".to_string(), json!(30_000));

    // Theme nest
    c.insert("theme".to_string(), json!({
        "Tab BG Color": "#333333",
        "Active Tab BG Color": "#2E7D32",
        "Highlighting Theme": "base16-ocean.dark"
    }));

    c
}

// Like len, but also includes nests, ignoring actual nest
pub fn nested_len(config: &BTreeMap<String, Value>) -> usize {
    let mut total = 0;
    for value in config.values() {
        total += count_values(value);
    }
    total
}

fn count_values(value: &Value) -> usize {
  match value {
      // Get the insides of a nested object
      Value::Object(map) => {
          let mut sub_total = 0;
          for v in map.values() {
              sub_total += count_values(v);
          }
          sub_total
      }

      _ => 1,
  }
}
