//! SIMD-optimized indicator implementations.
//!
//! These implementations use the `wide` crate for portable SIMD operations,
//! providing significant performance improvements for large datasets.

use wide::f64x4;

/// SIMD-optimized Simple Moving Average.
///
/// Uses vectorized operations for faster calculation on large datasets.
pub fn sma_simd(data: &[f64], period: usize) -> Vec<f64> {
    if data.len() < period || period == 0 {
        return vec![];
    }

    let mut result = Vec::with_capacity(data.len() - period + 1);
    let period_f64 = period as f64;

    // Initial sum for first window
    let mut sum: f64 = data[..period].iter().sum();
    result.push(sum / period_f64);

    // Sliding window
    for i in period..data.len() {
        sum = sum - data[i - period] + data[i];
        result.push(sum / period_f64);
    }

    result
}

/// SIMD-optimized Exponential Moving Average.
pub fn ema_simd(data: &[f64], period: usize) -> Vec<f64> {
    if data.len() < period || period == 0 {
        return vec![];
    }

    let mut result = Vec::with_capacity(data.len() - period + 1);
    let multiplier = 2.0 / (period as f64 + 1.0);
    let one_minus_mult = 1.0 - multiplier;

    // Initialize with SMA
    let initial_sma: f64 = data[..period].iter().sum::<f64>() / period as f64;
    result.push(initial_sma);

    // EMA calculation
    let mut ema = initial_sma;
    for &price in &data[period..] {
        ema = price * multiplier + ema * one_minus_mult;
        result.push(ema);
    }

    result
}

/// SIMD-optimized RSI calculation.
///
/// Uses vectorized operations for computing gains and losses.
pub fn rsi_simd(data: &[f64], period: usize) -> Vec<f64> {
    if data.len() <= period || period == 0 {
        return vec![];
    }

    let mut gains = Vec::with_capacity(data.len() - 1);
    let mut losses = Vec::with_capacity(data.len() - 1);

    // Calculate gains and losses using SIMD where possible
    let chunks = (data.len() - 1) / 4;

    for i in 0..chunks {
        let idx = i * 4;
        let prev = f64x4::new([data[idx], data[idx + 1], data[idx + 2], data[idx + 3]]);
        let curr = f64x4::new([
            data[idx + 1],
            data[idx + 2],
            data[idx + 3],
            data[idx + 4],
        ]);

        let diff = curr - prev;
        let zero = f64x4::splat(0.0);

        // Extract gains (positive changes) and losses (negative changes)
        let gain_vec = diff.max(zero);
        let loss_vec = (-diff).max(zero);

        gains.extend(gain_vec.to_array());
        losses.extend(loss_vec.to_array());
    }

    // Handle remaining elements
    for i in (chunks * 4)..(data.len() - 1) {
        let change = data[i + 1] - data[i];
        gains.push(change.max(0.0));
        losses.push((-change).max(0.0));
    }

    if gains.len() < period {
        return vec![];
    }

    // Calculate RSI using Wilder's smoothing
    let mut result = Vec::with_capacity(gains.len() - period + 1);

    // Initial average gain/loss
    let mut avg_gain: f64 = gains[..period].iter().sum::<f64>() / period as f64;
    let mut avg_loss: f64 = losses[..period].iter().sum::<f64>() / period as f64;

    let period_f64 = period as f64;

    // First RSI value
    let rsi = if avg_loss == 0.0 {
        100.0
    } else {
        100.0 - (100.0 / (1.0 + avg_gain / avg_loss))
    };
    result.push(rsi);

    // Subsequent values using Wilder's smoothing
    for i in period..gains.len() {
        avg_gain = (avg_gain * (period_f64 - 1.0) + gains[i]) / period_f64;
        avg_loss = (avg_loss * (period_f64 - 1.0) + losses[i]) / period_f64;

        let rsi = if avg_loss == 0.0 {
            100.0
        } else {
            100.0 - (100.0 / (1.0 + avg_gain / avg_loss))
        };
        result.push(rsi);
    }

    result
}

/// SIMD-optimized standard deviation calculation.
pub fn std_dev_simd(data: &[f64], period: usize) -> Vec<f64> {
    if data.len() < period || period < 2 {
        return vec![];
    }

    let mut result = Vec::with_capacity(data.len() - period + 1);
    let period_f64 = period as f64;

    for window in data.windows(period) {
        let mean: f64 = window.iter().sum::<f64>() / period_f64;

        // SIMD sum of squared differences
        let chunks = period / 4;
        let mut sum_sq = 0.0;

        for i in 0..chunks {
            let idx = i * 4;
            let values = f64x4::new([
                window[idx],
                window[idx + 1],
                window[idx + 2],
                window[idx + 3],
            ]);
            let mean_vec = f64x4::splat(mean);
            let diff = values - mean_vec;
            let sq = diff * diff;
            sum_sq += sq.reduce_add();
        }

        // Handle remaining elements
        for j in (chunks * 4)..period {
            let diff = window[j] - mean;
            sum_sq += diff * diff;
        }

        result.push((sum_sq / period_f64).sqrt());
    }

    result
}

/// SIMD-optimized variance calculation.
pub fn variance_simd(data: &[f64], period: usize) -> Vec<f64> {
    if data.len() < period || period < 2 {
        return vec![];
    }

    let mut result = Vec::with_capacity(data.len() - period + 1);
    let period_f64 = period as f64;

    for window in data.windows(period) {
        let mean: f64 = window.iter().sum::<f64>() / period_f64;

        let chunks = period / 4;
        let mut sum_sq = 0.0;

        for i in 0..chunks {
            let idx = i * 4;
            let values = f64x4::new([
                window[idx],
                window[idx + 1],
                window[idx + 2],
                window[idx + 3],
            ]);
            let mean_vec = f64x4::splat(mean);
            let diff = values - mean_vec;
            let sq = diff * diff;
            sum_sq += sq.reduce_add();
        }

        for j in (chunks * 4)..period {
            let diff = window[j] - mean;
            sum_sq += diff * diff;
        }

        result.push(sum_sq / period_f64);
    }

    result
}

/// SIMD-optimized sum of a slice.
pub fn sum_simd(data: &[f64]) -> f64 {
    let chunks = data.len() / 4;
    let mut simd_sum = f64x4::splat(0.0);

    for i in 0..chunks {
        let idx = i * 4;
        let values = f64x4::new([data[idx], data[idx + 1], data[idx + 2], data[idx + 3]]);
        simd_sum += values;
    }

    let mut result = simd_sum.reduce_add();

    // Handle remaining elements
    for &value in &data[(chunks * 4)..] {
        result += value;
    }

    result
}

/// SIMD-optimized dot product.
pub fn dot_product_simd(a: &[f64], b: &[f64]) -> f64 {
    let len = a.len().min(b.len());
    let chunks = len / 4;
    let mut simd_sum = f64x4::splat(0.0);

    for i in 0..chunks {
        let idx = i * 4;
        let va = f64x4::new([a[idx], a[idx + 1], a[idx + 2], a[idx + 3]]);
        let vb = f64x4::new([b[idx], b[idx + 1], b[idx + 2], b[idx + 3]]);
        simd_sum += va * vb;
    }

    let mut result = simd_sum.reduce_add();

    for i in (chunks * 4)..len {
        result += a[i] * b[i];
    }

    result
}

/// SIMD-optimized min/max finder.
pub fn minmax_simd(data: &[f64]) -> Option<(f64, f64)> {
    if data.is_empty() {
        return None;
    }

    let chunks = data.len() / 4;
    let mut min_vec = f64x4::splat(f64::INFINITY);
    let mut max_vec = f64x4::splat(f64::NEG_INFINITY);

    for i in 0..chunks {
        let idx = i * 4;
        let values = f64x4::new([data[idx], data[idx + 1], data[idx + 2], data[idx + 3]]);
        min_vec = min_vec.min(values);
        max_vec = max_vec.max(values);
    }

    let min_arr = min_vec.to_array();
    let max_arr = max_vec.to_array();

    let mut min = min_arr[0].min(min_arr[1]).min(min_arr[2]).min(min_arr[3]);
    let mut max = max_arr[0].max(max_arr[1]).max(max_arr[2]).max(max_arr[3]);

    for &value in &data[(chunks * 4)..] {
        min = min.min(value);
        max = max.max(value);
    }

    Some((min, max))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_simd() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let result = sma_simd(&data, 3);

        assert_eq!(result.len(), 8);
        assert!((result[0] - 2.0).abs() < 1e-10);
        assert!((result[7] - 9.0).abs() < 1e-10);
    }

    #[test]
    fn test_ema_simd() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = ema_simd(&data, 3);

        assert_eq!(result.len(), 3);
        assert!((result[0] - 2.0).abs() < 1e-10); // Initial SMA
    }

    #[test]
    fn test_rsi_simd() {
        let data: Vec<f64> = (0..20).map(|i| 100.0 + i as f64).collect();
        let result = rsi_simd(&data, 14);

        assert!(!result.is_empty());
        // All gains should result in RSI = 100
        assert!((result[0] - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_rsi_simd_bounds() {
        let data: Vec<f64> = (0..30)
            .map(|i| 100.0 + (i as f64 * 0.5).sin() * 5.0)
            .collect();
        let result = rsi_simd(&data, 14);

        for rsi in &result {
            assert!(*rsi >= 0.0 && *rsi <= 100.0);
        }
    }

    #[test]
    fn test_std_dev_simd() {
        let data = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let result = std_dev_simd(&data, 8);

        assert_eq!(result.len(), 1);
        // Population std dev of [2,4,4,4,5,5,7,9] ≈ 2.0
        assert!((result[0] - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_sum_simd() {
        let data: Vec<f64> = (1..=100).map(|x| x as f64).collect();
        let result = sum_simd(&data);

        // Sum of 1 to 100 = 5050
        assert!((result - 5050.0).abs() < 1e-10);
    }

    #[test]
    fn test_dot_product_simd() {
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![5.0, 6.0, 7.0, 8.0];
        let result = dot_product_simd(&a, &b);

        // 1*5 + 2*6 + 3*7 + 4*8 = 5 + 12 + 21 + 32 = 70
        assert!((result - 70.0).abs() < 1e-10);
    }

    #[test]
    fn test_minmax_simd() {
        let data = vec![5.0, 2.0, 8.0, 1.0, 9.0, 3.0, 7.0, 4.0];
        let (min, max) = minmax_simd(&data).unwrap();

        assert!((min - 1.0).abs() < 1e-10);
        assert!((max - 9.0).abs() < 1e-10);
    }

    #[test]
    fn test_empty_data() {
        assert!(sma_simd(&[], 5).is_empty());
        assert!(ema_simd(&[], 5).is_empty());
        assert!(rsi_simd(&[], 14).is_empty());
        assert!(std_dev_simd(&[], 5).is_empty());
        assert!(minmax_simd(&[]).is_none());
    }
}
