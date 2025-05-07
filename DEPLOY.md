# Code Beautifier Deployment Guide

This guide will help you deploy the Code Beautifier tool on your own server. This application formats and beautifies code with proper indentation.

## Quick Start with Docker Compose

The easiest way to deploy Code Beautifier is using the provided Docker Compose file:

```bash
# Clone the repository (make sure to rename it)
git clone https://github.com/ekzhang/rustpad.git code-beautifier
cd code-beautifier

# Build and run using Docker Compose
docker-compose up -d
```

This will start Code Beautifier on port 3030. You can access it at http://localhost:3030.

## Custom Deployment

If you want more control over the deployment process, you can use the custom Dockerfile:

```bash
# Build the container with the rebranded binary
docker build -t code-beautifier -f custom.Dockerfile .

# Run the container
docker run --rm -dp 3030:3030 code-beautifier
```

## Configuration Options

You can configure Code Beautifier using the following environment variables:

- `EXPIRY_DAYS`: How long to keep formatted documents in memory (default: 1 day)
- `SQLITE_URI`: Enable persistence with SQLite (example: `file:/data/db.sqlite`)
- `PORT`: The port to run on (default: 3030)
- `RUST_LOG`: Log level configuration

## Enabling Persistence

To enable document persistence between restarts:

1. Uncomment the volume and SQLite lines in `docker-compose.yml`
2. Create a data directory: `mkdir -p data`
3. Restart the container: `docker-compose down && docker-compose up -d`

## Customizing the UI Further

If you want to further customize the UI, you may need to modify:

1. The class/file names in the source code
2. Package dependencies in `package.json`
3. The colors and styling in the UI components

## Security Considerations

- The application doesn't include authentication by default
- Consider placing it behind a reverse proxy with authentication
- Use HTTPS in production environments 