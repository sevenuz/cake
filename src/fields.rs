//! free header fields: types and derived values from the store config
//! `fields`, e.g.
//!
//! ```hjson
//! fields: {
//!   tokens_in: { type: "number" }
//!   wh: { derive: "tokens_in * 0.00025 + tokens_out * 0.002" }
//! }
//! ```
//!
//! Derived values are computed when read, so a changed formula applies to
//! old items too. Formulas are evalexpr expressions over the item's fields
//! (strings and assignments allowed); a missing field counts as 0.

use evalexpr::{
    build_operator_tree, Context, ContextWithMutableVariables, DefaultNumericTypes, HashMapContext, Value as Ev,
};
use serde::Deserialize;
use serde_yaml_ng::{Mapping, Value};
use std::collections::BTreeMap;

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct FieldDef {
    /// number, list, date, duration or text (the default)
    #[serde(rename = "type")]
    pub kind: Option<String>,
    /// a formula; the field is computed, never stored
    pub derive: Option<String>,
}

pub type Defs = BTreeMap<String, FieldDef>;

/// a value given on the command line, checked against the field's type
pub fn typed(defs: &Defs, key: &str, raw: &str) -> Result<Value, String> {
    let def = defs.get(key);
    if def.is_some_and(|d| d.derive.is_some()) {
        return Err(format!("'{}' is derived (store config fields), not set", key));
    }
    Ok(match def.and_then(|d| d.kind.as_deref()) {
        Some("number") => match crate::item::field_value(raw) {
            v @ Value::Number(_) => v,
            _ => return Err(format!("'{}' is a number, not '{}'", key, raw)),
        },
        Some("date") => Value::String(crate::util::parse_date(raw).map_err(|e| format!("'{}': {}", key, e))?),
        Some("duration") => {
            let s = crate::util::parse_duration(raw).map_err(|_| format!("'{}' is a duration like 2h30m", key))?;
            Value::String(crate::util::timestamp_to_hms(s))
        }
        Some("text") => Value::String(raw.to_string()),
        Some("list") => crate::item::field_value(raw),
        Some(other) => return Err(format!("'{}' has the unknown type '{}' (number, list, date, duration, text)", key, other)),
        None => crate::item::field_value(raw),
    })
}

/// true if `key` holds a list: by its type, else by its current value
pub fn is_list(defs: &Defs, key: &str, current: Option<&Value>) -> bool {
    match defs.get(key).and_then(|d| d.kind.as_deref()) {
        Some("list") => true,
        Some(_) => false,
        None => matches!(current, Some(Value::Sequence(_))),
    }
}

fn to_ev(v: &Value) -> Option<Ev<DefaultNumericTypes>> {
    match v {
        Value::Number(n) => n.as_i64().map(Ev::Int).or_else(|| n.as_f64().map(Ev::Float)),
        Value::Bool(b) => Some(Ev::Boolean(*b)),
        Value::String(s) => Some(Ev::String(s.clone())),
        Value::Sequence(l) => Some(Ev::Int(l.len() as i64)),
        _ => None,
    }
}

fn to_json(v: &Ev<DefaultNumericTypes>) -> serde_json::Value {
    match v {
        Ev::Int(i) => (*i).into(),
        Ev::Float(f) => serde_json::Number::from_f64(*f).map_or(serde_json::Value::Null, Into::into),
        Ev::Boolean(b) => (*b).into(),
        Ev::String(s) => s.clone().into(),
        Ev::Empty => serde_json::Value::Null,
        other => other.to_string().into(),
    }
}

/// the derived fields of an item with these stored `fields`; a derived
/// field may use others in any order (evaluated until nothing changes, at
/// most once per derived field). A formula that fails gives "?" and its
/// error.
pub fn derive(defs: &Defs, fields: &Mapping) -> (BTreeMap<String, serde_json::Value>, Vec<String>) {
    let derived: Vec<(&String, &String)> = defs.iter().filter_map(|(k, d)| d.derive.as_ref().map(|f| (k, f))).collect();
    let mut last: BTreeMap<String, Ev<DefaultNumericTypes>> = BTreeMap::new();
    let mut errors: BTreeMap<String, String> = BTreeMap::new();
    for _ in 0..derived.len().max(1) {
        // a fresh context per pass: assignments in formulas do not pile up
        let mut ctx = HashMapContext::<DefaultNumericTypes>::new();
        for (k, v) in fields {
            if let (Some(k), Some(v)) = (k.as_str(), to_ev(v)) {
                let _ = ctx.set_value(k.to_string(), v);
            }
        }
        for (k, v) in &last {
            let _ = ctx.set_value(k.clone(), v.clone());
        }
        let mut next = BTreeMap::new();
        errors.clear();
        for (key, formula) in &derived {
            let result = build_operator_tree::<DefaultNumericTypes>(formula).and_then(|tree| {
                let missing: Vec<String> =
                    tree.iter_variable_identifiers().filter(|v| ctx.get_value(v).is_none()).map(String::from).collect();
                for v in missing {
                    let _ = ctx.set_value(v, Ev::Int(0));
                }
                tree.eval_with_context_mut(&mut ctx)
            });
            match result {
                Ok(v) => {
                    let _ = ctx.set_value((*key).clone(), v.clone());
                    next.insert((*key).clone(), v);
                }
                Err(e) => {
                    errors.insert((*key).clone(), format!("field {}: {}", key, e));
                }
            }
        }
        let done = next == last;
        last = next;
        if done {
            break;
        }
    }
    let mut out: BTreeMap<String, serde_json::Value> = last.iter().map(|(k, v)| (k.clone(), to_json(v))).collect();
    for k in errors.keys() {
        out.insert(k.clone(), "?".into());
    }
    (out, errors.into_values().collect())
}

fn json_to_ev(v: &serde_json::Value) -> Option<Ev<DefaultNumericTypes>> {
    match v {
        serde_json::Value::Number(n) => n.as_i64().map(Ev::Int).or_else(|| n.as_f64().map(Ev::Float)),
        serde_json::Value::Bool(b) => Some(Ev::Boolean(*b)),
        serde_json::Value::String(s) => Some(Ev::String(s.clone())),
        serde_json::Value::Array(a) => Some(Ev::Int(a.len() as i64)),
        _ => None,
    }
}

fn number(v: &Ev<DefaultNumericTypes>) -> Option<f64> {
    match v {
        Ev::Int(i) => Some(*i as f64),
        Ev::Float(f) if f.is_finite() => Some(*f),
        _ => None,
    }
}

/// the formulas `named` (name, formula), in order, over `vars`: each result
/// is a variable for the later ones; missing variables are 0, a formula
/// that fails or gives no number is 0
pub fn eval_all(named: &[(String, String)], vars: &serde_json::Map<String, serde_json::Value>) -> Vec<f64> {
    let mut ctx = HashMapContext::<DefaultNumericTypes>::new();
    for (k, v) in vars {
        if let Some(v) = json_to_ev(v) {
            let _ = ctx.set_value(k.clone(), v);
        }
    }
    let mut out = vec![];
    for (name, formula) in named {
        let v = build_operator_tree::<DefaultNumericTypes>(formula)
            .and_then(|tree| {
                let missing: Vec<String> =
                    tree.iter_variable_identifiers().filter(|v| ctx.get_value(v).is_none()).map(String::from).collect();
                for v in missing {
                    let _ = ctx.set_value(v, Ev::Int(0));
                }
                tree.eval_with_context_mut(&mut ctx)
            })
            .ok()
            .and_then(|v| number(&v))
            .unwrap_or(0.0);
        let _ = ctx.set_value(name.clone(), Ev::Float(v));
        out.push(v);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defs(text: &str) -> Defs {
        deser_hjson::from_str(text).unwrap()
    }

    #[test]
    fn derived_fields_use_stored_ones_and_each_other() {
        let d = defs(r#"{ tokens_in: { type: "number" }, wh: { derive: "tokens_in * 0.5 + tokens_out" }, co2: { derive: "wh * 2" }, tag: { derive: "\"v\" + str::from(tokens_in)" } }"#);
        let fields: Mapping = serde_yaml_ng::from_str("tokens_in: 10\n").unwrap();
        let (out, errors) = derive(&d, &fields);
        assert!(errors.is_empty(), "{:?}", errors);
        assert_eq!(out["wh"], serde_json::json!(5.0));
        assert_eq!(out["co2"], serde_json::json!(10.0), "uses wh although it sorts first");
        assert_eq!(out["tag"], serde_json::json!("v10"));
    }

    #[test]
    fn a_broken_formula_gives_a_question_mark() {
        let d = defs(r#"{ x: { derive: "1 +" } }"#);
        let (out, errors) = derive(&d, &Mapping::new());
        assert_eq!(out["x"], serde_json::json!("?"));
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn named_formulas_build_on_each_other() {
        let vars = serde_json::json!({"a": 2, "b": 3.5, "s": "x"});
        let named = vec![("t".to_string(), "a + b".to_string()), ("u".to_string(), "t * 2".to_string()), ("bad".to_string(), "s".to_string())];
        assert_eq!(eval_all(&named, vars.as_object().unwrap()), vec![5.5, 11.0, 0.0]);
    }

    #[test]
    fn typed_values_are_checked() {
        let d = defs(r#"{ n: { type: "number" }, due2: { type: "date" }, w: { derive: "1" } }"#);
        assert_eq!(typed(&d, "n", "3").unwrap(), Value::from(3));
        assert!(typed(&d, "n", "x").is_err());
        assert!(typed(&d, "due2", "2026-12-01").is_ok());
        assert!(typed(&d, "w", "1").is_err());
        assert_eq!(typed(&d, "free", "true").unwrap(), Value::Bool(true));
    }
}
