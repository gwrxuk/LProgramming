# Solana DEX Trading Bot

A production-level trading bot for Solana DEXs with advanced features for LP management, price monitoring, and CEX integration.

## Features

- DEX Integration
  - Raydium and Jupiter DEX support
  - LP creation and management
  - Range rebalancing
  - Fee harvesting
  - Capital rotation strategies

- Price Feeds
  - Jupiter aggregator integration
  - Pyth Network oracle support
  - Switchboard oracle integration

- Simulation Tools
  - Volume simulation across multiple wallets
  - Randomized trade sizes and timings
  - Performance analysis

- CEX Integration
  - Binance
  - Bybit
  - OKX

- Monitoring & Analytics
  - Performance metrics logging
  - Dashboard data preparation
  - Real-time monitoring

## Prerequisites

- Rust 1.70 or higher
- Solana CLI tools
- PostgreSQL database
- API keys for CEX integrations

## Configuration

1. Copy `.env.example` to `.env`
2. Update the configuration with your API keys and settings
3. Configure your database connection

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```

## Testing

```bash
cargo test
```

## Project Structure

```
src/
├── dex/           # DEX integration modules
├── oracles/       # Price feed integrations
├── cex/          # CEX integration modules
├── models/       # Data models and types
├── utils/        # Utility functions
├── config/       # Configuration management
├── metrics/      # Performance monitoring
└── simulation/   # Trading simulation tools
```

## Security

- Never commit API keys or private keys
- Use environment variables for sensitive data
- Implement proper error handling and rate limiting
- Follow security best practices for key management

## License

MIT License

## Contributing

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request 