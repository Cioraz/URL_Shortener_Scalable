# High Performance URL Shortener

## System Design
![Architecture drawio 1](https://github.com/user-attachments/assets/26589b0b-8c49-4655-ab90-74bd6097b373)

## Performance Tests
![image](https://github.com/user-attachments/assets/b30d3036-91b9-4b18-87b4-d88dcb576870)

### Test Configuration
- 4 threads
- 100 concurrent connections to server
- Test duration: 30 seconds

## 🚀 Features

- URL shortening with custom and auto-generated short URLs
- High-performance Rust backend with Warp framework
- Redis-based storage for fast access
- Prometheus metrics integration
- Docker containerization
- Nginx reverse proxy
- Frontend interface
- Health monitoring
- Supports 100+ concurrent connections

## 🏗️ Architecture

The project consists of the following components:

- **Backend**: Rust-based API server using Warp framework
- **Frontend**: Web interface
- **Redis**: For storing URL mappings
- **Nginx**: Reverse proxy and load balancer
- **Prometheus**: Metrics collection (optional)
- **Grafana**: Metrics visualization (optional)

## 🛠️ Technology Stack

- **Backend**: Rust + Warp
- **Database**: Redis
- **Containerization**: Docker + Docker Compose
- **Reverse Proxy**: Nginx
- **Monitoring**: Prometheus + Grafana
- **API Testing**: curl/Postman

## 📋 Prerequisites

- Docker and Docker Compose
- Rust (for local development)
- Redis
- Git

## 🚀 Getting Started

1. Clone the repository:
```bash
git clone https://github.com/yourusername/URL_Shortener_Scalable.git
cd URL_Shortener_Scalable
```

2. Set up environment variables:
```bash
cp .env.example .env
# Edit .env file with your configurations
```

3. Build and run using Docker Compose:
```bash
docker-compose up --build
```

## 🔌 API Endpoints

### Generate Short URL
```
POST /generate_url
Header: API-Key: your_api_key
Content-Type: application/json
Body: {"url": "https://example.com"}
```

### Custom Short URL
```
POST /custom_url
Header: API-Key: your_api_key
Content-Type: application/json
Body: {"url": "https://example.com", "custom_path": "my-custom-path"}
```

### Resolve Short URL
```
GET /dns_resolver/:short_url
```

### Health Check
```
GET /ping
```

### Metrics
```
GET /metrics
```

## 🔍 Monitoring

The service includes Prometheus metrics for:
- Total URL generation requests
- Request duration metrics
- Custom metrics for error rates
- Real-time performance monitoring

## 🔒 Security

- API Key authentication for URL generation
- Rate limiting (via Nginx)
- Input validation
- CORS configuration

## 🚀 Production Deployment

For production deployment:
1. Update the `.env` file with production values
2. Enable HTTPS in Nginx configuration
3. Configure proper monitoring
4. Set up backups for Redis
5. Configure proper logging

## 📈 Performance Metrics

- Handles 100+ concurrent connections
- Multi-threaded processing (4 threads)
- High availability through containerization
- Load balanced through Nginx
- Redis-based caching for fast lookups

## 📝 License

[MIT License](LICENSE)

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
