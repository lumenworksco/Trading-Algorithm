#!/usr/bin/env python3
"""Generate realistic mock stock data for backtesting."""

import csv
import random
import math
from datetime import datetime, timedelta

def generate_stock_data(
    symbol: str,
    start_date: datetime,
    end_date: datetime,
    initial_price: float,
    volatility: float = 0.02,
    drift: float = 0.0001,
    output_file: str = None
):
    """
    Generate realistic OHLCV data using geometric Brownian motion.

    Args:
        symbol: Stock symbol
        start_date: Start date
        end_date: End date
        initial_price: Starting price
        volatility: Daily volatility (standard deviation)
        drift: Daily drift (mean return)
        output_file: Output CSV file path
    """
    if output_file is None:
        output_file = f"data/{symbol.lower()}_daily.csv"

    data = []
    current_date = start_date
    price = initial_price

    while current_date <= end_date:
        # Skip weekends
        if current_date.weekday() >= 5:
            current_date += timedelta(days=1)
            continue

        # Generate daily return using GBM
        daily_return = drift + volatility * random.gauss(0, 1)

        # Calculate OHLC
        open_price = price

        # Intraday volatility
        intraday_vol = volatility * 0.5

        # Generate high/low with some randomness
        high_move = abs(random.gauss(0, intraday_vol))
        low_move = abs(random.gauss(0, intraday_vol))

        # Apply daily return to get close
        close_price = open_price * (1 + daily_return)

        # High is max of open/close plus some upside
        high_price = max(open_price, close_price) * (1 + high_move)

        # Low is min of open/close minus some downside
        low_price = min(open_price, close_price) * (1 - low_move)

        # Ensure high >= max(open, close) and low <= min(open, close)
        high_price = max(high_price, open_price, close_price)
        low_price = min(low_price, open_price, close_price)

        # Generate volume (higher on volatile days)
        base_volume = random.randint(1000000, 5000000)
        vol_multiplier = 1 + abs(daily_return) * 10
        volume = int(base_volume * vol_multiplier)

        # Format timestamp
        timestamp = current_date.strftime("%Y-%m-%dT09:30:00")

        data.append({
            'timestamp': timestamp,
            'open': round(open_price, 2),
            'high': round(high_price, 2),
            'low': round(low_price, 2),
            'close': round(close_price, 2),
            'volume': volume
        })

        # Next day starts at this day's close
        price = close_price
        current_date += timedelta(days=1)

    # Write to CSV
    with open(output_file, 'w', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=['timestamp', 'open', 'high', 'low', 'close', 'volume'])
        writer.writeheader()
        writer.writerows(data)

    print(f"Generated {len(data)} bars for {symbol} -> {output_file}")
    return data


def main():
    # Set random seed for reproducibility
    random.seed(42)

    # Generate data for 2023
    start = datetime(2023, 1, 1)
    end = datetime(2023, 12, 31)

    # Generate data for multiple symbols with different characteristics
    stocks = [
        # symbol, initial_price, volatility, drift
        ('AAPL', 150.0, 0.02, 0.0003),   # Apple - moderate vol, slight uptrend
        ('GOOGL', 100.0, 0.025, 0.0002), # Google - higher vol
        ('MSFT', 250.0, 0.018, 0.0004),  # Microsoft - lower vol, stronger uptrend
        ('SPY', 400.0, 0.012, 0.0002),   # S&P 500 ETF - low vol
        ('QQQ', 300.0, 0.015, 0.0003),   # Nasdaq ETF
        ('TSLA', 200.0, 0.04, 0.0001),   # Tesla - very high vol
    ]

    for symbol, price, vol, drift in stocks:
        generate_stock_data(
            symbol=symbol,
            start_date=start,
            end_date=end,
            initial_price=price,
            volatility=vol,
            drift=drift,
            output_file=f"data/{symbol.lower()}_daily.csv"
        )

    # Also generate a combined file
    print("\nGenerating combined multi-symbol file...")

    all_data = []
    for symbol, price, vol, drift in stocks:
        random.seed(42 + hash(symbol) % 1000)  # Different seed per symbol
        current_date = start
        current_price = price

        while current_date <= end:
            if current_date.weekday() >= 5:
                current_date += timedelta(days=1)
                continue

            daily_return = drift + vol * random.gauss(0, 1)
            open_price = current_price
            close_price = open_price * (1 + daily_return)

            intraday_vol = vol * 0.5
            high_move = abs(random.gauss(0, intraday_vol))
            low_move = abs(random.gauss(0, intraday_vol))

            high_price = max(open_price, close_price) * (1 + high_move)
            low_price = min(open_price, close_price) * (1 - low_move)

            base_volume = random.randint(1000000, 5000000)
            volume = int(base_volume * (1 + abs(daily_return) * 10))

            all_data.append({
                'timestamp': current_date.strftime("%Y-%m-%dT09:30:00"),
                'symbol': symbol,
                'open': round(open_price, 2),
                'high': round(high_price, 2),
                'low': round(low_price, 2),
                'close': round(close_price, 2),
                'volume': volume
            })

            current_price = close_price
            current_date += timedelta(days=1)

    # Sort by timestamp then symbol
    all_data.sort(key=lambda x: (x['timestamp'], x['symbol']))

    with open('data/multi_symbol_daily.csv', 'w', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=['timestamp', 'symbol', 'open', 'high', 'low', 'close', 'volume'])
        writer.writeheader()
        writer.writerows(all_data)

    print(f"Generated combined file with {len(all_data)} total bars -> data/multi_symbol_daily.csv")


if __name__ == '__main__':
    main()
