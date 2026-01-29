//! Timeframe definitions for market data.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Timeframe for bars/candles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Timeframe {
    /// 1 minute bars
    #[serde(rename = "1m")]
    Minute1,
    /// 5 minute bars
    #[serde(rename = "5m")]
    Minute5,
    /// 15 minute bars
    #[serde(rename = "15m")]
    Minute15,
    /// 30 minute bars
    #[serde(rename = "30m")]
    Minute30,
    /// 1 hour bars
    #[serde(rename = "1h")]
    Hour1,
    /// 4 hour bars
    #[serde(rename = "4h")]
    Hour4,
    /// Daily bars
    #[serde(rename = "1d")]
    #[default]
    Daily,
    /// Weekly bars
    #[serde(rename = "1w")]
    Weekly,
    /// Monthly bars
    #[serde(rename = "1M")]
    Monthly,
}

impl Timeframe {
    /// Get the duration of the timeframe in seconds.
    pub fn as_secs(&self) -> u64 {
        match self {
            Timeframe::Minute1 => 60,
            Timeframe::Minute5 => 300,
            Timeframe::Minute15 => 900,
            Timeframe::Minute30 => 1800,
            Timeframe::Hour1 => 3600,
            Timeframe::Hour4 => 14400,
            Timeframe::Daily => 86400,
            Timeframe::Weekly => 604800,
            Timeframe::Monthly => 2592000, // Approximate (30 days)
        }
    }

    /// Get the duration of the timeframe in milliseconds.
    pub fn as_millis(&self) -> u64 {
        self.as_secs() * 1000
    }

    /// Check if this is an intraday timeframe.
    pub fn is_intraday(&self) -> bool {
        matches!(
            self,
            Timeframe::Minute1
                | Timeframe::Minute5
                | Timeframe::Minute15
                | Timeframe::Minute30
                | Timeframe::Hour1
                | Timeframe::Hour4
        )
    }

    /// Get all available timeframes.
    pub fn all() -> &'static [Timeframe] {
        &[
            Timeframe::Minute1,
            Timeframe::Minute5,
            Timeframe::Minute15,
            Timeframe::Minute30,
            Timeframe::Hour1,
            Timeframe::Hour4,
            Timeframe::Daily,
            Timeframe::Weekly,
            Timeframe::Monthly,
        ]
    }
}

impl fmt::Display for Timeframe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Timeframe::Minute1 => "1m",
            Timeframe::Minute5 => "5m",
            Timeframe::Minute15 => "15m",
            Timeframe::Minute30 => "30m",
            Timeframe::Hour1 => "1h",
            Timeframe::Hour4 => "4h",
            Timeframe::Daily => "1d",
            Timeframe::Weekly => "1w",
            Timeframe::Monthly => "1M",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Timeframe {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "1m" | "1min" | "minute" => Ok(Timeframe::Minute1),
            "5m" | "5min" => Ok(Timeframe::Minute5),
            "15m" | "15min" => Ok(Timeframe::Minute15),
            "30m" | "30min" => Ok(Timeframe::Minute30),
            "1h" | "1hour" | "hour" => Ok(Timeframe::Hour1),
            "4h" | "4hour" => Ok(Timeframe::Hour4),
            "1d" | "day" | "daily" => Ok(Timeframe::Daily),
            "1w" | "week" | "weekly" => Ok(Timeframe::Weekly),
            "1M" | "month" | "monthly" => Ok(Timeframe::Monthly),
            _ => Err(format!("Invalid timeframe: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeframe_duration() {
        assert_eq!(Timeframe::Minute1.as_secs(), 60);
        assert_eq!(Timeframe::Hour1.as_secs(), 3600);
        assert_eq!(Timeframe::Daily.as_secs(), 86400);
    }

    #[test]
    fn test_timeframe_parse() {
        assert_eq!(Timeframe::from_str("1m").unwrap(), Timeframe::Minute1);
        assert_eq!(Timeframe::from_str("1d").unwrap(), Timeframe::Daily);
        assert_eq!(Timeframe::from_str("daily").unwrap(), Timeframe::Daily);
    }

    #[test]
    fn test_timeframe_display() {
        assert_eq!(Timeframe::Minute1.to_string(), "1m");
        assert_eq!(Timeframe::Daily.to_string(), "1d");
    }

    #[test]
    fn test_is_intraday() {
        assert!(Timeframe::Minute1.is_intraday());
        assert!(Timeframe::Hour4.is_intraday());
        assert!(!Timeframe::Daily.is_intraday());
        assert!(!Timeframe::Weekly.is_intraday());
    }
}
