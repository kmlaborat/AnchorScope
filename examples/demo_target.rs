// AnchorScope v1.1.0 Demo Target File
// This file contains intentional TODO comments and similar-looking functions
// for demonstrating the new auto-label and label-based editing features.

use std::collections::HashMap;

/// Calculates the sum of all items in a vector
pub fn calculate_total(items: Vec<f64>) -> f64 {
    // TODO: Add input validation (reject negative numbers)
    items.iter().sum()
}

/// Calculates the average of all items in a vector
pub fn calculate_average(items: Vec<f64>) -> f64 {
    // TODO: Handle empty input to avoid division by zero
    let sum = items.iter().sum::<f64>();
    sum / items.len() as f64
}

/// Looks up a value in a HashMap, returning a default
pub fn get_or_default<'a, K: std::hash::Hash + Eq, V: Clone>(
    map: &'a HashMap<K, V>,
    key: &K,
    default: V,
) -> V {
    // TODO: Consider using Entry API for better performance
    map.get(key).cloned().unwrap_or(default)
}

/// Converts a string to uppercase
pub fn to_upper(s: String) -> String {
    s.to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_total() {
        assert_eq!(calculate_total(vec![1.0, 2.0, 3.0]), 6.0);
    }

    #[test]
    fn test_calculate_average() {
        assert_eq!(calculate_average(vec![2.0, 4.0]), 3.0);
    }
}
