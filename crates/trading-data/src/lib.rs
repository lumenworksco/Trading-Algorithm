//! Data sources for trading.

mod cache;
mod csv_source;

pub use cache::DataCache;
pub use csv_source::CsvDataSource;

use trading_core::error::DataError;
use trading_core::types::{Bar, Timeframe};

/// Load bars from a CSV file.
pub async fn load_csv(
    path: &str,
    symbol: &str,
    timeframe: Timeframe,
) -> Result<Vec<Bar>, DataError> {
    let source = CsvDataSource::new(path)?;
    source.load_all(symbol, timeframe).await
}
