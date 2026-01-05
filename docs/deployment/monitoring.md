# Monitoring & Observability Guide

Comprehensive guide for monitoring, logging, and observability of Cobalt Stack in production environments.

## Overview

Effective monitoring and observability are critical for:
- **Detecting issues** before users notice them
- **Diagnosing problems** quickly when they occur
- **Understanding system behavior** under load
- **Planning capacity** for future growth
- **Meeting SLAs** and uptime guarantees

**Three Pillars of Observability**:
1. **Metrics**: Quantitative measurements (request rate, CPU usage)
2. **Logs**: Detailed event records (errors, requests, transactions)
3. **Traces**: Request paths through distributed systems (future)

## Health Monitoring

### Health Check Endpoint

**Backend Health Endpoint**: `/health`

**Response Format**:
```json
{
  "status": "healthy",
  "timestamp": "2025-10-27T12:00:00Z"
}
```

**Status Codes**:
- `200 OK`: All systems operational
- `503 Service Unavailable`: System unhealthy (database down, etc.)

**Future Enhancements**:
```json
{
  "status": "healthy",
  "timestamp": "2025-10-27T12:00:00Z",
  "checks": {
    "database": "ok",
    "redis": "ok",
    "disk_space": "ok",
    "memory": "ok"
  },
  "version": "1.0.0",
  "uptime_seconds": 86400
}
```

### Docker Health Checks

**Backend Health Check** (docker-compose.prod.yml):
```yaml
healthcheck:
  test: ["CMD-SHELL", "curl -f http://localhost:3000/health || exit 1"]
  interval: 30s       # Check every 30 seconds
  timeout: 10s        # Timeout after 10 seconds
  retries: 3          # Mark unhealthy after 3 failures
  start_period: 40s   # Grace period for startup
```

**PostgreSQL Health Check**:
```yaml
healthcheck:
  test: ["CMD-SHELL", "pg_isready -U postgres"]
  interval: 10s
  timeout: 5s
  retries: 5
```

**Redis Health Check**:
```yaml
healthcheck:
  test: ["CMD", "redis-cli", "ping"]
  interval: 10s
  timeout: 3s
  retries: 5
```

**View Health Status**:
```bash
# All services
docker compose ps

# Specific service health
docker inspect --format='{{.State.Health.Status}}' cobalt-backend-prod

# Health check logs
docker inspect --format='{{range .State.Health.Log}}{{.Output}}{{end}}' cobalt-backend-prod
```

### External Monitoring

**Uptime Monitoring Services**:

#### 1. UptimeRobot (Free Tier)
- 50 monitors free
- 5-minute check intervals
- Email/SMS alerts
- Public status page

**Setup**:
```
1. Sign up at https://uptimerobot.com
2. Add HTTP(s) monitor: https://yourdomain.com/health
3. Set alert contacts (email, Slack, PagerDuty)
4. Configure: Check every 5 minutes, alert on 2 failures
```

#### 2. Pingdom (Commercial)
- 1-minute check intervals
- Multi-location checks
- Detailed response time graphs
- Real user monitoring

#### 3. Custom Health Check Script

**Simple Cron-based Monitoring**:
```bash
#!/bin/bash
# /opt/cobalt-stack/scripts/health-check.sh

ENDPOINT="https://yourdomain.com/health"
ALERT_EMAIL="admin@yourdomain.com"
SLACK_WEBHOOK="https://hooks.slack.com/services/YOUR/WEBHOOK/URL"

# Check health endpoint
RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" --max-time 10 $ENDPOINT)

if [ $RESPONSE -ne 200 ]; then
    # Log failure
    echo "$(date): Health check failed - HTTP $RESPONSE" >> /var/log/cobalt-health.log

    # Send email alert
    echo "Cobalt Stack health check failed: HTTP $RESPONSE" | \
      mail -s "ALERT: Cobalt Stack Down" $ALERT_EMAIL

    # Send Slack notification
    curl -X POST $SLACK_WEBHOOK \
      -H "Content-Type: application/json" \
      -d "{\"text\":\"🚨 Cobalt Stack health check failed: HTTP $RESPONSE\"}"

    # Exit with error
    exit 1
fi

# Success
exit 0
```

**Schedule with Cron** (every 5 minutes):
```bash
# Add to crontab
crontab -e

# Check every 5 minutes
*/5 * * * * /opt/cobalt-stack/scripts/health-check.sh
```

## Logging Configuration

### Application Logging

**Backend Logging** (Rust tracing):

**Log Levels**:
- `trace`: Very detailed (function entry/exit, variable values)
- `debug`: Debugging information (SQL queries, API calls)
- `info`: General informational messages (startup, shutdown, requests)
- `warn`: Warning conditions (deprecated features, non-critical errors)
- `error`: Error conditions (failed requests, exceptions)

**Configuration** (RUST_LOG environment variable):
```bash
# Production (recommended)
RUST_LOG=info

# Development
RUST_LOG=debug

# Debug specific modules
RUST_LOG=cobalt_stack_backend=debug,tower_http=info,sea_orm=warn

# Trace everything (very verbose)
RUST_LOG=trace
```

**Log Format** (Structured JSON for production):
```json
{
  "timestamp": "2025-10-27T12:00:00.123Z",
  "level": "info",
  "target": "cobalt_stack_backend::handlers::auth",
  "message": "User login successful",
  "fields": {
    "user_id": "123e4567-e89b-12d3-a456-426614174000",
    "email": "user@example.com",
    "request_id": "abc123"
  }
}
```

**Implementation** (backend/src/main.rs):
```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// Development: Human-readable
tracing_subscriber::registry()
    .with(EnvFilter::from_default_env())
    .with(tracing_subscriber::fmt::layer())
    .init();

// Production: JSON format (future)
tracing_subscriber::registry()
    .with(EnvFilter::from_default_env())
    .with(tracing_subscriber::fmt::layer().json())
    .init();
```

### Docker Logging

**Log Drivers**:

#### 1. json-file (Default)
**Configuration** (docker-compose.prod.yml):
```yaml
services:
  backend:
    logging:
      driver: "json-file"
      options:
        max-size: "20m"     # Max 20MB per log file
        max-file: "5"       # Keep 5 rotated files
                            # Total: 100MB max per service
```

**View Logs**:
```bash
# All services
docker compose -f docker-compose.prod.yml logs

# Follow logs (tail -f)
docker compose -f docker-compose.prod.yml logs -f backend

# Last 100 lines
docker compose -f docker-compose.prod.yml logs --tail=100 backend

# Since timestamp
docker compose -f docker-compose.prod.yml logs --since 2025-10-27T10:00:00 backend

# Parse JSON logs with jq
docker compose -f docker-compose.prod.yml logs backend | jq -r '.message'
```

#### 2. syslog (Centralized Logging)
**Configuration**:
```yaml
services:
  backend:
    logging:
      driver: "syslog"
      options:
        syslog-address: "tcp://logserver.example.com:514"
        tag: "cobalt-backend"
```

#### 3. Fluentd (Log Aggregation)
**Configuration**:
```yaml
services:
  backend:
    logging:
      driver: "fluentd"
      options:
        fluentd-address: "localhost:24224"
        tag: "cobalt.backend"
```

### Log Aggregation (ELK Stack)

**ELK Stack**: Elasticsearch + Logstash + Kibana

**Docker Compose Addition**:
```yaml
services:
  elasticsearch:
    image: elasticsearch:8.10.0
    environment:
      - discovery.type=single-node
      - "ES_JAVA_OPTS=-Xms512m -Xmx512m"
    volumes:
      - elasticsearch_data:/usr/share/elasticsearch/data
    ports:
      - "9200:9200"

  logstash:
    image: logstash:8.10.0
    volumes:
      - ./logstash.conf:/usr/share/logstash/pipeline/logstash.conf
    depends_on:
      - elasticsearch

  kibana:
    image: kibana:8.10.0
    ports:
      - "5601:5601"
    environment:
      ELASTICSEARCH_HOSTS: http://elasticsearch:9200
    depends_on:
      - elasticsearch

volumes:
  elasticsearch_data:
```

**Logstash Configuration** (logstash.conf):
```
input {
  tcp {
    port => 5000
    codec => json
  }
}

filter {
  json {
    source => "message"
  }
  date {
    match => ["timestamp", "ISO8601"]
  }
}

output {
  elasticsearch {
    hosts => ["elasticsearch:9200"]
    index => "cobalt-logs-%{+YYYY.MM.dd}"
  }
}
```

**Application Configuration**:
```yaml
services:
  backend:
    logging:
      driver: "syslog"
      options:
        syslog-address: "tcp://logstash:5000"
        syslog-format: "rfc5424"
```

**Access Kibana**: http://localhost:5601

## Error Tracking

### Application Errors

**Backend Error Logging**:
```rust
// Automatic error logging in handlers
pub async fn handler() -> Result<Response, AuthError> {
    match perform_operation().await {
        Ok(result) => Ok(result),
        Err(e) => {
            tracing::error!("Operation failed: {}", e);
            Err(e)
        }
    }
}
```

**Error Response Format**:
```json
{
  "error": "InvalidCredentials",
  "message": "Email or password is incorrect",
  "request_id": "abc123",
  "timestamp": "2025-10-27T12:00:00Z"
}
```

### Sentry Integration (Future)

**Sentry**: Error tracking and monitoring service

**Setup**:
```bash
# Add to Cargo.toml
sentry = "0.32"
sentry-tracing = "0.32"
```

**Configuration**:
```rust
// backend/src/main.rs
let _guard = sentry::init((
    "https://examplePublicKey@o0.ingest.sentry.io/0",
    sentry::ClientOptions {
        release: Some(env!("CARGO_PKG_VERSION").into()),
        environment: Some("production".into()),
        ..Default::default()
    },
));

// Integrate with tracing
let sentry_layer = sentry_tracing::layer();
tracing_subscriber::registry()
    .with(sentry_layer)
    .with(tracing_subscriber::fmt::layer())
    .init();
```

**Error Capture**:
```rust
// Automatic error capture
tracing::error!("Database connection failed: {}", error);

// Manual error capture
sentry::capture_error(&error);
```

**Benefits**:
- Error grouping and deduplication
- Stack traces and context
- Release tracking
- User impact analysis
- Alerting and notifications

## Performance Monitoring

### Response Time Monitoring

**Backend Request Logging**:
```rust
// Middleware for request duration
use tower_http::trace::TraceLayer;

let app = Router::new()
    .layer(TraceLayer::new_for_http()
        .on_request(|request: &Request, _span: &Span| {
            tracing::info!("request started: {} {}", request.method(), request.uri());
        })
        .on_response(|response: &Response, latency: Duration, _span: &Span| {
            tracing::info!("request completed: status={}, latency={:?}",
                response.status(), latency);
        })
    );
```

**Log Output**:
```
2025-10-27T12:00:00 INFO request started: POST /api/auth/login
2025-10-27T12:00:00 INFO request completed: status=200, latency=45ms
```

**Analysis**:
```bash
# Extract response times from logs
docker compose logs backend | grep "request completed" | \
  grep -oP "latency=\K[0-9]+ms" | sort -n | tail -10

# Average response time (requires jq and jq-based parsing)
docker compose logs backend --since 1h | grep "latency" | \
  awk '{print $NF}' | sed 's/ms//' | \
  awk '{sum+=$1; count++} END {print sum/count "ms"}'
```

### Database Query Monitoring

**Enable Slow Query Log** (PostgreSQL):
```sql
-- Log queries taking longer than 1 second
ALTER SYSTEM SET log_min_duration_statement = 1000;
SELECT pg_reload_conf();

-- View slow queries
SELECT * FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;
```

**Enable pg_stat_statements**:
```sql
-- Enable extension
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

-- View query statistics
SELECT
  calls,
  total_exec_time,
  mean_exec_time,
  query
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;
```

**Monitor from Application**:
```bash
# Connect to PostgreSQL
docker compose exec postgres psql -U postgres -d cobalt_prod

# Run query analysis
\x on
SELECT * FROM pg_stat_statements ORDER BY mean_exec_time DESC LIMIT 5;
```

### Resource Monitoring

**Docker Container Stats**:
```bash
# Real-time stats (all containers)
docker stats

# Specific container
docker stats cobalt-backend-prod

# Export to file
docker stats --no-stream > container-stats.txt
```

**Output**:
```
CONTAINER           CPU %    MEM USAGE / LIMIT     MEM %    NET I/O           BLOCK I/O
cobalt-backend      2.5%     512MiB / 2GiB         25%      1.2GB / 456MB     10MB / 5MB
cobalt-postgres     1.2%     1GiB / 2GiB           50%      500MB / 200MB     50MB / 20MB
cobalt-redis        0.1%     50MiB / 512MiB        10%      100MB / 50MB      1MB / 500KB
```

**System Resource Monitoring**:
```bash
# CPU and memory
htop

# Disk usage
df -h
du -sh /var/lib/docker

# Disk I/O
iotop

# Network
iftop
```

## Metrics Collection (Prometheus)

### Prometheus Setup

**Docker Compose Addition**:
```yaml
services:
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--storage.tsdb.retention.time=30d'

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3100:3000"
    environment:
      GF_SECURITY_ADMIN_PASSWORD: admin
    volumes:
      - grafana_data:/var/lib/grafana
    depends_on:
      - prometheus

volumes:
  prometheus_data:
  grafana_data:
```

**Prometheus Configuration** (prometheus.yml):
```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'backend'
    static_configs:
      - targets: ['backend:3000']
    metrics_path: '/metrics'

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']

  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']
```

### Backend Metrics Endpoint (Future)

**Add Prometheus Metrics**:
```rust
// Cargo.toml
prometheus = "0.13"
lazy_static = "1.4"

// backend/src/metrics.rs
use prometheus::{Counter, Histogram, Registry, TextEncoder, Encoder};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref HTTP_REQUESTS_TOTAL: Counter = Counter::new(
        "http_requests_total",
        "Total HTTP requests"
    ).unwrap();

    pub static ref HTTP_REQUEST_DURATION: Histogram = Histogram::new(
        "http_request_duration_seconds",
        "HTTP request duration"
    ).unwrap();

    pub static ref REGISTRY: Registry = {
        let r = Registry::new();
        r.register(Box::new(HTTP_REQUESTS_TOTAL.clone())).unwrap();
        r.register(Box::new(HTTP_REQUEST_DURATION.clone())).unwrap();
        r
    };
}

// Metrics endpoint
pub async fn metrics() -> Response<String> {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = String::new();
    encoder.encode_utf8(&metric_families, &mut buffer).unwrap();

    Response::builder()
        .status(200)
        .header("Content-Type", "text/plain")
        .body(buffer)
        .unwrap()
}
```

**Instrument Handlers**:
```rust
pub async fn handler() -> Result<Response> {
    let timer = HTTP_REQUEST_DURATION.start_timer();
    HTTP_REQUESTS_TOTAL.inc();

    let result = perform_operation().await;

    timer.observe_duration();
    result
}
```

### Grafana Dashboards

**Access Grafana**: http://localhost:3100 (default: admin/admin)

**Add Prometheus Data Source**:
1. Configuration → Data Sources
2. Add Prometheus: http://prometheus:9090

**Import Dashboards**:
- Node Exporter Full: 1860
- Docker Container Metrics: 193
- PostgreSQL Database: 9628

**Custom Dashboard Panels**:
- HTTP Request Rate: `rate(http_requests_total[5m])`
- Average Response Time: `rate(http_request_duration_seconds_sum[5m]) / rate(http_request_duration_seconds_count[5m])`
- Error Rate: `rate(http_errors_total[5m])`

## Alerting

### Alert Manager (Prometheus)

**Alert Rules** (prometheus-rules.yml):
```yaml
groups:
  - name: cobalt_alerts
    interval: 30s
    rules:
      # High error rate
      - alert: HighErrorRate
        expr: rate(http_errors_total[5m]) > 0.05
        for: 5m
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} errors/sec"

      # High response time
      - alert: HighResponseTime
        expr: rate(http_request_duration_seconds_sum[5m]) / rate(http_request_duration_seconds_count[5m]) > 1
        for: 5m
        annotations:
          summary: "High response time"
          description: "Average response time is {{ $value }}s"

      # Database connection errors
      - alert: DatabaseConnectionFailed
        expr: up{job="postgres"} == 0
        for: 2m
        annotations:
          summary: "Database connection failed"
          description: "PostgreSQL is unreachable"

      # High CPU usage
      - alert: HighCPUUsage
        expr: container_cpu_usage_seconds_total > 0.8
        for: 10m
        annotations:
          summary: "High CPU usage"
          description: "CPU usage is {{ $value }}%"
```

**Alert Manager Configuration** (alertmanager.yml):
```yaml
global:
  smtp_smarthost: 'smtp.sendgrid.net:587'
  smtp_from: 'alerts@yourdomain.com'
  smtp_auth_username: 'apikey'
  smtp_auth_password: '<sendgrid-api-key>'

route:
  group_by: ['alertname']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  receiver: 'email-alert'

receivers:
  - name: 'email-alert'
    email_configs:
      - to: 'admin@yourdomain.com'
        headers:
          Subject: 'Cobalt Stack Alert: {{ .GroupLabels.alertname }}'

  - name: 'slack-alert'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/YOUR/WEBHOOK/URL'
        channel: '#alerts'
        title: 'Cobalt Stack Alert'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'
```

### Simple Alerting Script

**Disk Space Alert**:
```bash
#!/bin/bash
# /opt/cobalt-stack/scripts/disk-space-alert.sh

THRESHOLD=80
USAGE=$(df -h / | tail -1 | awk '{print $5}' | sed 's/%//')

if [ $USAGE -gt $THRESHOLD ]; then
    echo "Disk space critical: ${USAGE}% used" | \
      mail -s "ALERT: Low Disk Space" admin@yourdomain.com
fi
```

**Schedule**:
```bash
# Check every hour
0 * * * * /opt/cobalt-stack/scripts/disk-space-alert.sh
```

## Best Practices

### Logging

1. **Structured Logging**: Use JSON format in production for easy parsing
2. **Log Levels**: Use appropriate levels (info for production, debug for development)
3. **Context**: Include request IDs, user IDs, timestamps
4. **Sensitive Data**: Never log passwords, tokens, credit cards
5. **Retention**: Rotate logs, keep 30-90 days
6. **Performance**: Avoid excessive logging (impacts performance and costs)

### Monitoring

1. **Health Checks**: Monitor every service (app, database, cache)
2. **External Monitoring**: Use external service to detect network/DNS issues
3. **Alert Fatigue**: Set thresholds to avoid too many alerts
4. **Actionable Alerts**: Every alert should require action
5. **Escalation**: Define escalation paths (email → SMS → phone call)

### Metrics

1. **Golden Signals**: Monitor latency, traffic, errors, saturation
2. **Business Metrics**: Track user signups, logins, transactions
3. **Baseline**: Establish normal behavior to detect anomalies
4. **Dashboards**: Create role-specific dashboards (ops, dev, business)

## Troubleshooting Common Issues

### High CPU Usage
**Diagnosis**:
```bash
docker stats cobalt-backend-prod
```

**Solutions**:
- Optimize slow queries (check `pg_stat_statements`)
- Add database indexes
- Implement caching (Redis)
- Scale horizontally (multiple backend instances)

### High Memory Usage
**Diagnosis**:
```bash
docker inspect cobalt-backend-prod | grep Memory
```

**Solutions**:
- Check for memory leaks (Rust is generally safe)
- Reduce connection pool size
- Increase container memory limit
- Enable query result pagination

### Slow Requests
**Diagnosis**:
```bash
# Extract slow requests from logs
docker compose logs backend | grep "latency" | awk '{print $NF}' | sort -n
```

**Solutions**:
- Profile application (add timing logs)
- Optimize database queries
- Enable database query caching
- Use Redis for frequent reads

### Log Storage Full
**Diagnosis**:
```bash
du -sh /var/lib/docker/containers
```

**Solutions**:
- Enable log rotation in docker-compose
- Reduce log retention period
- Clean old logs: `docker system prune --volumes`
- Move logs to external storage (S3, CloudWatch)

## References

- [Prometheus Documentation](https://prometheus.io/docs)
- [Grafana Documentation](https://grafana.com/docs)
- [ELK Stack Guide](https://www.elastic.co/what-is/elk-stack)
- [Sentry Documentation](https://docs.sentry.io)
- [Docker Logging Best Practices](https://docs.docker.com/config/containers/logging)
- [12-Factor App: Logs](https://12factor.net/logs)
