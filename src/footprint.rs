//! a rough, deliberately conservative estimate of energy, CO2e and water
//! behind the tokens of an item. Real values depend on hardware, batch
//! size, data center and grid; the defaults lean high and are meant for
//! orders of magnitude, every number is configurable (config `footprint`).

use crate::item::Usage;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// Wh per 1000 tokens of each kind, before model factor and PUE
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Rates {
    pub input: f64,
    pub output: f64,
    pub cache_read: f64,
    pub cache_write: f64,
}

impl Default for Rates {
    fn default() -> Self {
        // output (decoding) dominates; prefill and cache writes cost about a
        // tenth of it per token, cache reads a hundredth
        Rates { input: 0.3, output: 3.0, cache_read: 0.03, cache_write: 0.3 }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Footprint {
    /// Wh per 1000 tokens for a large model
    pub wh_per_1k: Rates,
    /// multiplier by model name part, the longest matching key wins, "*" otherwise
    pub model_factor: BTreeMap<String, f64>,
    /// data center overhead (power usage effectiveness)
    pub pue: f64,
    /// grid carbon intensity, g CO2e per kWh
    pub co2_g_per_kwh: f64,
    /// water per kWh: cooling on site plus power generation
    pub water_l_per_kwh: f64,
}

impl Default for Footprint {
    fn default() -> Self {
        let factor = [("*", 1.0), ("opus", 1.0), ("sonnet", 0.5), ("haiku", 0.2)];
        Footprint {
            wh_per_1k: Rates::default(),
            model_factor: factor.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            pue: 1.2,
            co2_g_per_kwh: 450.0,
            water_l_per_kwh: 2.0,
        }
    }
}

static CONFIG: OnceLock<Footprint> = OnceLock::new();

pub fn init(f: Footprint) {
    let _ = CONFIG.set(f);
}

fn config() -> &'static Footprint {
    CONFIG.get_or_init(Footprint::default)
}

fn factor(f: &Footprint, model: &str) -> f64 {
    f.model_factor
        .iter()
        .filter(|(k, _)| *k != "*" && model.contains(k.as_str()))
        .max_by_key(|(k, _)| k.len())
        .or_else(|| f.model_factor.get_key_value("*"))
        .map_or(1.0, |(_, v)| *v)
}

/// Wh of `usage` under `f`
fn wh(f: &Footprint, usage: &BTreeMap<String, Usage>) -> f64 {
    let r = &f.wh_per_1k;
    usage
        .iter()
        .map(|(model, u)| {
            let raw = u.input as f64 * r.input
                + u.output as f64 * r.output
                + u.cache_read as f64 * r.cache_read
                + u.cache_write as f64 * r.cache_write;
            raw / 1000.0 * factor(f, model)
        })
        .sum::<f64>()
        * f.pue
}

/// {wh, co2_g, water_l}, null without usage
pub fn estimate(usage: &BTreeMap<String, Usage>) -> serde_json::Value {
    if usage.is_empty() {
        return serde_json::Value::Null;
    }
    let f = config();
    let wh = wh(f, usage);
    serde_json::json!({
        "wh": (wh * 10.0).round() / 10.0,
        "co2_g": (wh / 1000.0 * f.co2_g_per_kwh * 10.0).round() / 10.0,
        "water_l": (wh / 1000.0 * f.water_l_per_kwh * 1000.0).round() / 1000.0,
    })
}

fn human(n: u64) -> String {
    match n {
        0..=999 => n.to_string(),
        1_000..=999_999 => format!("{:.1}k", n as f64 / 1e3),
        _ => format!("{:.1}M", n as f64 / 1e6),
    }
}

/// "tokens 1.2M (claude-opus-5-5) ~45 Wh, ~20 g CO2e, ~0.09 l water"
pub fn describe(usage: &BTreeMap<String, Usage>) -> String {
    let f = config();
    let total: u64 = usage.values().map(|u| u.total()).sum();
    let models: Vec<&str> = usage.keys().map(|k| k.as_str()).collect();
    let wh = wh(f, usage);
    format!(
        "tokens {} ({}) ~{:.0} Wh, ~{:.0} g CO2e, ~{:.2} l water",
        human(total),
        models.join(", "),
        wh,
        wh / 1000.0 * f.co2_g_per_kwh,
        wh / 1000.0 * f.water_l_per_kwh
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_scales_with_tokens_model_and_pue() {
        let f = Footprint::default();
        let u = |m: &str| {
            [(m.to_string(), Usage { input: 1000, output: 1000, cache_read: 10_000, cache_write: 1000 })].into()
        };
        // (0.3 + 3.0 + 0.3 + 0.3) Wh * 1.2 PUE
        assert!((wh(&f, &u("claude-opus-5-5")) - 4.68).abs() < 1e-9);
        assert!((wh(&f, &u("claude-haiku-5-5")) - 4.68 * 0.2).abs() < 1e-9);
        assert!((wh(&f, &u("some-local-model")) - 4.68).abs() < 1e-9, "unknown models use *");
        assert!(describe(&u("claude-opus-5-5")).starts_with("tokens 13.0k (claude-opus-5-5) ~5 Wh"));
    }
}
