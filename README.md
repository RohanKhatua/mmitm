# Meet Me in the Middle (MMITM) Backend

A Rust backend service that recommends optimal meeting points for groups based on their GPS coordinates and preferences. Built according to the Software Requirements Specification for fair, travel-time-optimized venue recommendations.

## Features

- **Fair Meeting Point Calculation**: Automatically calculates optimal search centers and radii
  - 2 participants: Midpoint with smart radius calculation
  - 3+ participants: Centroid with maximum distance consideration
- **Modern Google APIs**: Uses the latest Google Places API (New) and Distance Matrix API
- **Category-based Filtering**: Support for all Google Places API Table A types
- **Travel Time Optimization**: Uses Google Distance Matrix API for accurate travel times
- **Fairness-based Ranking**: Prioritizes venues that minimize travel time imbalance
- **High Performance**: Built with Rust and Axum for concurrent request handling
- **Comprehensive Testing**: Unit and integration tests included

## Quick Start

### Prerequisites

- Rust 1.70+ installed
- Google Maps Platform API key (see [GOOGLE_SETUP.md](GOOGLE_SETUP.md))

### Installation

1. Clone and setup:

```bash
git clone <repository-url>
cd mmitm
cp .env.example .env
```

2. Add your Google Maps API key to `.env`:

```env
GOOGLE_MAPS_API_KEY=your_actual_api_key_here
```

3. Run the server:

```bash
cargo run
```

4. Test the API:

```bash
./test_api.sh
```

## API Usage

### Basic Request

```bash
curl -X POST http://localhost:3000/recommendations \
  -H "Content-Type: application/json" \
  -d '{
    "participants": [
      { "lat": 12.9716, "lng": 77.5946 },
      { "lat": 12.2958, "lng": 76.6394 }
    ],
    "categories": ["restaurant", "cafe"],
    "limit": 5
  }'
```

### Response

```json
[
	{
		"name": "Cafe Central",
		"address": "123 MG Road, Bangalore",
		"lat": 12.9345,
		"lng": 77.6102,
		"rating": 4.5,
		"reviews": 250,
		"google_url": "https://maps.google.com/...",
		"travel_times": [35, 40],
		"category": "cafe",
		"price_level": 2
	}
]
```

See [API_DOCS.md](API_DOCS.md) for complete API documentation.

## Supported Categories

The application supports all Google Places API Table A types including:

- **Food & Drink**: `restaurant`, `cafe`, `bar`, `bakery`
- **Entertainment**: `movie_theater`, `museum`, `art_gallery`
- **Recreation**: `park`, `zoo`, `stadium`, `gym`
- **Shopping**: `shopping_mall`, `store`, `book_store`

[Complete list in GOOGLE_SETUP.md](GOOGLE_SETUP.md)

## Algorithm Details

### Search Center Calculation

- **2 participants**: Midpoint between coordinates
- **3+ participants**: Geographic centroid of all coordinates

### Search Radius

- **2 participants**: `min((distance / 2 + 1km), 5km)`
- **3+ participants**: `min((max_pairwise_distance / 2 + 2km), 50km)`

### Ranking Algorithm

1. **Primary**: Fairness score (minimize max-min travel time difference)
2. **Secondary**: Total travel time (minimize sum of all travel times)
3. **Penalty**: For 2 participants, penalize venues with >10min imbalance

## Development

### Running Tests

```bash
cargo test
```

### Debug Mode

```bash
RUST_LOG=debug cargo run
```

### Code Coverage

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out html
```

## Deployment

### Docker

```bash
# Build
docker build -t mmitm .

# Run
docker run -p 3000:3000 -e GOOGLE_MAPS_API_KEY=your_key mmitm
```

### Docker Compose

```bash
echo "GOOGLE_MAPS_API_KEY=your_key" > .env
docker-compose up
```

### Production Considerations

1. **API Key Security**: Use environment variables, never commit keys
2. **Rate Limiting**: Implement request throttling for production
3. **Monitoring**: Set up logging and Google API quota monitoring
4. **Caching**: Consider caching results for frequently requested areas
5. **Load Balancing**: Use multiple instances behind a load balancer

## Performance

- **Response Time**: <2 seconds for 90% of requests (excluding Google API latency)
- **Concurrency**: Handles up to 500 concurrent requests
- **Memory Usage**: ~10MB base memory footprint
- **API Efficiency**: Batches requests to minimize Google API quota usage

## Cost Optimization

The application is designed to minimize Google API costs:

- **Batched Requests**: Places details fetched in batches
- **Smart Filtering**: Pre-filters results before expensive Distance Matrix calls
- **Radius Optimization**: Calculates optimal search radius to avoid over-fetching

Estimated cost per 1000 recommendations: $0.50-1.50 (depending on result count and travel matrix size)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure `cargo test` passes
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Client App    │    │   MMITM Backend  │    │  Google Maps    │
│                 │    │                  │    │     APIs        │
├─────────────────┤    ├──────────────────┤    ├─────────────────┤
│ Mobile/Web App  │◄──►│ Axum Web Server  │◄──►│ Places API      │
│ JSON Requests   │    │ Rust Backend     │    │ Distance Matrix │
│                 │    │ Fair Algorithms  │    │ Place Details   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Support

- **Documentation**: See API_DOCS.md and GOOGLE_SETUP.md
- **Issues**: Create GitHub issues for bugs or feature requests
- **Performance**: Monitor logs and Google Cloud Console for quota usage
