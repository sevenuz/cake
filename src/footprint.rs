//! a rough, deliberately conservative estimate of energy, CO2e and water
//! behind the tokens of an item. Real values depend on hardware, batch
//! size, data center and grid; the defaults lean high and are meant for
//! orders of magnitude, every number is configurable (config `footprint`);
//! the sources of the defaults are in config::TEMPLATE.

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
        Rates { input: 0.25, output: 2.0, cache_read: 0.025, cache_write: 0.25 }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Footprint {
    /// Wh per 1000 tokens for a large model
    pub wh_per_1k: Rates,
    /// multiplier by model name part, the longest matching key wins, "*" otherwise
    pub model_factor: BTreeMap<String, f64>,
    /// server around the GPUs (CPU, memory, network)
    pub server_overhead: f64,
    /// data center overhead (power usage effectiveness)
    pub pue: f64,
    /// grid carbon intensity, g CO2e per kWh
    pub co2_g_per_kwh: f64,
    /// water on site per kWh of IT energy (WUE)
    pub wue_l_per_kwh: f64,
    /// water at the power plants per kWh drawn
    pub ewif_l_per_kwh: f64,
}

impl Default for Footprint {
    fn default() -> Self {
        let factor = [("*", 1.0), ("opus", 1.0), ("sonnet", 0.5), ("haiku", 0.2)];
        Footprint {
            wh_per_1k: Rates::default(),
            model_factor: factor.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            server_overhead: 1.3,
            pue: 1.2,
            co2_g_per_kwh: 226.4,
            wue_l_per_kwh: 1.8,
            ewif_l_per_kwh: 2.0,
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

/// (Wh at the servers, Wh drawn by the data center) of `usage` under `f`
fn energy(f: &Footprint, usage: &BTreeMap<String, Usage>) -> (f64, f64) {
    let r = &f.wh_per_1k;
    let raw = usage
        .iter()
        .map(|(model, u)| {
            let raw = u.input as f64 * r.input
                + u.output as f64 * r.output
                + u.cache_read as f64 * r.cache_read
                + u.cache_write as f64 * r.cache_write;
            raw / 1000.0 * factor(f, model)
        })
        .sum::<f64>()
        * f.server_overhead;
    (raw, raw * f.pue)
}

/// Wh drawn, g CO2e, litres of water
fn totals(f: &Footprint, usage: &BTreeMap<String, Usage>) -> (f64, f64, f64) {
    let (it, drawn) = energy(f, usage);
    (drawn, drawn / 1000.0 * f.co2_g_per_kwh, it / 1000.0 * f.wue_l_per_kwh + drawn / 1000.0 * f.ewif_l_per_kwh)
}

/// (Wh drawn, g CO2e, litres of water) of `usage` under the config
pub fn amounts(usage: &BTreeMap<String, Usage>) -> (f64, f64, f64) {
    totals(config(), usage)
}

/// {wh, co2_g, water_l}, null without usage
pub fn estimate(usage: &BTreeMap<String, Usage>) -> serde_json::Value {
    if usage.is_empty() {
        return serde_json::Value::Null;
    }
    let (wh, co2, water) = totals(config(), usage);
    serde_json::json!({
        "wh": (wh * 10.0).round() / 10.0,
        "co2_g": (co2 * 10.0).round() / 10.0,
        "water_l": (water * 1000.0).round() / 1000.0,
    })
}

pub fn human(n: u64) -> String {
    match n {
        0..=999 => n.to_string(),
        1_000..=999_999 => format!("{:.1}k", n as f64 / 1e3),
        _ => format!("{:.1}M", n as f64 / 1e6),
    }
}

/// "tokens 1.2M (claude-opus-5-5) ~45 Wh, ~20 g CO2e, ~0.09 l water"
pub fn describe(usage: &BTreeMap<String, Usage>) -> String {
    let total: u64 = usage.values().map(|u| u.total()).sum();
    let models: Vec<&str> = usage.keys().map(|k| k.as_str()).collect();
    let (wh, co2, water) = totals(config(), usage);
    format!("tokens {} ({}) ~{:.0} Wh, ~{:.0} g CO2e, ~{:.2} l water", human(total), models.join(", "), wh, co2, water)
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
        // (0.25 + 2.0 + 0.25 + 0.25) Wh * 1.3 server * 1.2 PUE
        let drawn = |m: &str| totals(&f, &u(m)).0;
        assert!((drawn("claude-opus-5-5") - 2.75 * 1.3 * 1.2).abs() < 1e-9);
        assert!((drawn("claude-haiku-5-5") - 2.75 * 1.3 * 1.2 * 0.2).abs() < 1e-9);
        assert!((drawn("some-local-model") - drawn("claude-opus-5-5")).abs() < 1e-9, "unknown models use *");
        let (wh, co2, water) = totals(&f, &u("m"));
        assert!((co2 - wh / 1000.0 * 226.4).abs() < 1e-9);
        assert!((water - (2.75 * 1.3 / 1000.0 * 1.8 + wh / 1000.0 * 2.0)).abs() < 1e-9);
        assert!(describe(&u("claude-opus-5-5")).starts_with("tokens 13.0k (claude-opus-5-5) ~4 Wh"));
    }
}
