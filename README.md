# High Performance URL Shortener

## System Architecture
```ascii
                        +------------------+
                        |                  |
                        |   Load Balancer  |  (Nginx :80)
                        |                  |
                        +--+------+------+-+
                           |      |      |
                     +-----+      |      +-----+
                     |            |            |
              +------v--+   +-----v---+   +---v------+
              |Backend 1|   |Backend 2|   |Backend 3 |
              |(15555)  |   |(15556)  |   |(15557)  |
              +----+----+   +----+----+   +----+----+
                   |             |             |
                   |             |             |
                   +------+------+------+------+
                          |             |
                    +-----v-------------v-----+
                    |                         |
                    |      Redis (6379)       |
                    |                         |
                    +-------------------------+
```

## 🚀 Features

- URL shortening with custom and auto-generated short URLs
- High-performance Rust backend with Warp framework
- Redis-based storage for fast access
- Docker containerization
- Nginx reverse proxy
- Frontend interface
- Supports 100+ concurrent connections

## 🏗️ Architecture Components

### Load Balancer (Nginx)
- Port: 80
- Load balancing strategy: Consistent hashing
- Connection pooling: 300 keepalive connections
- Health checks enabled
```nginx
upstream backend_servers {
    hash $request_uri consistent;
    server backend1:8000 max_fails=3 fail_timeout=30s max_conns=50000;
    server backend2:8000 max_fails=3 fail_timeout=30s max_conns=50000;
    server backend3:8000 max_fails=3 fail_timeout=30s max_conns=50000;
    keepalive 300;
}
```

### 🖥️ Backend Servers (Rust + Warp)

| Server        | Host Port |
|---------------|-----------|
| **Backend 1** | 15555     |
| **Backend 2** | 15556     |
| **Backend 3** | 15557     |

- Internal container port: 8000
- Environment configuration through .env file

## 🛠️ Technology Stack

| Category            | Tools                         |
|---------------------|-------------------------------|
| **Backend**         | Rust + Warp                   |
| **Database**        | Redis                         |
| **Containerization**| Docker & Docker Compose       |
| **Reverse Proxy**   | Nginx                         |
| **API Testing**     | curl / Postman                |
| **Stress Testing**  | wrk                           |


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

## 🔒 Security

- API Key authentication for URL generation
- Rate limiting (via Nginx)
- Input validation
- CORS configuration

## 📈 Performance Metrics

- Handles 10,000+ concurrent connections
- Multi-threaded processing
- High availability through containerization
- Load balanced through Nginx
- Redis-based storage for fast lookups

## Performance Tests
We tested our entire setup using wrk_post.lua file where we sent requests to the server and these are the results..

### Setup
- Load Testing Tool: wrk
- Test Duration: 30 seconds
- Threads: 6, 7, 8 respectively
- Connections: 5000, 7000, 8000 respectively
- Endpoint Tested: /generate_url
- Custom Load Script: wrk_post.lua
Benchmark tests were performed on the direct backend as well as on a load-balanced setup to compare performance.

![Tested using 6 threads and 5000 connections each](image.png)

---

![Tested using 7 threads and 7000 connections each](image-1.png)

---

![Tested using 8 threads and 8000 connections each](image-2.png)

## 🚀 Performance Results

### Load Balanced Performance (Port 80)
```bash
wrk -t7 -c7000 -d30s -s wrk_post.lua http://127.0.0.1:80/generate_url
```
Results:
- Throughput: 11,504.66 req/sec
- Latency: 602.38ms average
- Transfer: 2.53 MB/sec
- Total Requests: 346,110 in 30.08s

### Direct Backend Performance (Port 15555)
```bash
wrk -t7 -c7000 -d30s -s wrk_post.lua http://127.0.0.1:15555/generate_url
```
Results:
- Throughput: 10,263.40 req/sec
- Latency: 671.89ms average
- Transfer: 1.81 MB/sec
- Total Requests: 308,189 in 30.03s

## Load Distribution
- Each backend handles ~2,333 connections under load
- Automatic failover if any backend fails
- Even distribution through consistent hashing
- Connection pooling reduces overhead

## Future Progressions

- **Integrate Cassandra for Persistent Storage:**  
  Migrate URL mappings to a highly available Cassandra database to ensure persistent, distributed, and scalable storage across multiple nodes.

- **Use Redis for Caching Only:**  
  Limit Redis usage strictly for caching hot entries to reduce backend database lookups, improving overall response times and reducing system load.

- **Lower Overall Latency:**  
  Optimize data access paths by introducing an efficient caching strategy and minimizing network hops, thus reducing request latency across the system.

- **Integrate Kubernetes for Better Scaling:**  
  Deploy the application on a Kubernetes cluster to enable dynamic scaling based on traffic, provide automatic failover, and ensure high availability across pods and services.



## 📝 License

[MIT License](LICENSE)


