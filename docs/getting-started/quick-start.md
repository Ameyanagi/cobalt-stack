# Quick Start Guide

Get Cobalt Stack running in 5 minutes.

## Prerequisites

- Docker and Docker Compose
- Git
- Modern web browser

## Installation Steps

### 1. Clone the Repository

```bash
git clone https://github.com/yourusername/cobalt-stack.git
cd cobalt-stack
```

### 2. Configure Environment

```bash
# Copy environment template
cp .env.example .env

# Edit .env with your settings (optional for local development)
nano .env
```

### 3. Start Services

```bash
# Start all services with Docker Compose
docker-compose up -d

# Or use Make command
make up
```

This will start:
- PostgreSQL database (port 5432)
- Rust backend API (port 8080)
- React frontend (port 3000)
- Traefik reverse proxy (ports 80/443)

### 4. Verify Installation

Open your browser and navigate to:

- **Frontend**: http://localhost:3000
- **API**: http://localhost:8080/api/health
- **Traefik Dashboard**: http://localhost:8080/dashboard/

### 5. Create Your First User

```bash
# Access the backend container
docker-compose exec backend bash

# Run user creation script
cargo run --bin create-user -- --email admin@example.com --password admin123
```

### 6. Login

1. Navigate to http://localhost:3000/login
2. Use credentials: `admin@example.com` / `admin123`
3. You're in!

## What's Next?

Now that you have Cobalt Stack running:

- **[Explore the Project Structure](./project-structure.md)** - Understand the codebase
- **[Read the Full Installation Guide](./installation.md)** - Detailed setup instructions
- **[Check the API Documentation](../api/README.md)** - Learn about available endpoints
- **[Build Your First Feature](../guides/README.md)** - Follow our tutorials

## Quick Commands

```bash
# View logs
docker-compose logs -f

# Restart services
docker-compose restart

# Stop services
docker-compose down

# Rebuild after code changes
docker-compose up -d --build

# Run tests
make test

# View API documentation
make docs
```

## Development Workflow

```bash
# Start development with hot reload
make dev

# Run backend only
cd backend && cargo run

# Run frontend only
cd frontend && npm run dev

# Run database migrations
make migrate

# Create new migration
make migration name=add_users_table
```

## Troubleshooting

### Port Already in Use

```bash
# Find process using port 3000
lsof -i :3000

# Kill the process
kill -9 <PID>
```

### Database Connection Failed

```bash
# Check if PostgreSQL is running
docker-compose ps

# Restart database
docker-compose restart postgres

# Check logs
docker-compose logs postgres
```

### Frontend Not Loading

```bash
# Clear npm cache and reinstall
cd frontend
rm -rf node_modules .next
npm install
npm run dev
```

## Common Issues

| Issue | Solution |
|-------|----------|
| Port 3000 already in use | Change `FRONTEND_PORT` in `.env` |
| Database connection error | Verify `DATABASE_URL` in `.env` |
| API not responding | Check `docker-compose logs backend` |
| Build failing | Run `docker-compose down -v` and rebuild |

## Getting Help

- **[Full Installation Guide](./installation.md)** - Detailed setup
- **[Troubleshooting Guide](../troubleshooting/README.md)** - Common issues
- **[API Documentation](../api/README.md)** - API reference
- **[Contributing Guide](../contributing/README.md)** - How to contribute

## Next Steps

Ready to dive deeper?

1. **[Understand the Architecture](./project-structure.md)**
2. **[Explore the API](../api/README.md)**
3. **[Build Your First Feature](../guides/README.md)**
4. **[Deploy to Production](../deployment/README.md)**

---

**Need help?** Check our [Troubleshooting Guide](../troubleshooting/README.md) or open an issue on GitHub.
