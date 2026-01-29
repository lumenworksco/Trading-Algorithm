//! Data sources for trading.

mod csv_source;
mod cache;

pub use csv_source::CsvDataSource;
pub use cache::DataCache;

use trading_core::types::{Bar, Timeframe};
use trading_core::error::DataError;


/// Load bars from a CSV file.
pub async fn load_csv(
    path: &str,
    symbol: &str,
    timeframe: Timeframe,
) -> Result<Vec<Bar>, DataError> {
    let source = CsvDataSource::new(path)?;
    source.load_all(symbol, timeframe).await
}
