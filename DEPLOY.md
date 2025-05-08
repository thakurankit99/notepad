# Code Beautifier Deployment Guide

This guide will help you deploy the Code Beautifier tool on your own server. This application formats and beautifies code with proper indentation.

## Quick Start with Docker Compose

The easiest way to deploy Code Beautifier is using the provided Docker Compose file:

```bash
# Clone the repository (make sure to rename it)
git clone https://github.com/ekzhang/rustpad.git code-beautifier
cd code-beautifier

# Set your PostgreSQL connection string as an environment variable
export POSTGRES_URI=your_postgres_connection_string

# Build and run using Docker Compose
docker-compose up -d
```

This will start Code Beautifier on port 3030. You can access it at http://localhost:3030.

## Custom Deployment

If you want more control over the deployment process, you can use the custom Dockerfile:

```bash
# Build the container with the rebranded binary
docker build -t code-beautifier -f custom.Dockerfile .

# Run the container with your PostgreSQL connection string
docker run --rm -dp 3030:3030 -e POSTGRES_URI=your_postgres_connection_string code-beautifier
```

## Configuration Options

You can configure Code Beautifier using the following environment variables:

- `EXPIRY_DAYS`: How long to keep formatted documents in memory (default: 1 day)
- `POSTGRES_URI`: PostgreSQL connection string (required for persistence)
- `PORT`: The port to run on (default: 3030)
- `RUST_LOG`: Log level configuration
- `KEEP_ALIVE_INTERVAL_SECS`: Interval in seconds between keep-alive pings (default: 30, set to 0 to disable)
- `KEEP_ALIVE_URL`: (Optional) Custom URL to ping (by default, the app auto-detects its URL)

## PostgreSQL Setup

The application uses PostgreSQL for document persistence. You'll need to provide a valid PostgreSQL connection string in the format:

```
postgres://username:password@hostname:port/database?sslmode=require
```

**IMPORTANT:** Never store database credentials directly in your codebase or configuration files that will be committed to version control. Always use environment variables or secret management tools for sensitive information.

## Keep-Alive Service

The application includes a built-in keep-alive service that can ping itself at regular intervals to prevent it from going idle on free-tier hosting services like Render. This is useful to ensure your application stays responsive even during periods of inactivity.

The service automatically detects its own URL using the following logic:
1. First tries to use the `RENDER_EXTERNAL_URL` environment variable (provided by Render)
2. Then tries to construct a URL using the `RENDER_EXTERNAL_HOSTNAME` (provided by Render)
3. Falls back to the manually specified `KEEP_ALIVE_URL` if provided
4. If none of the above work, defaults to `http://localhost:{PORT}`

To configure the keep-alive service:
- Set `KEEP_ALIVE_INTERVAL_SECS` to the desired interval between pings (e.g., 30 for every 30 seconds)
- Set to 0 to disable the keep-alive service
- Optionally set `KEEP_ALIVE_URL` to override the auto-detection

When properly configured, the server logs will show:
- "Starting keep-alive service with interval of X seconds" at startup
- "Keep-alive service will ping URL: {detected-url}" showing the detected URL
- "Performing keep-alive ping to URL" at each ping interval
- "Keep-alive ping successful" when the ping succeeds

## Render Deployment

When deploying to Render:

1. Use the `custom.Dockerfile` directly
2. Add your PostgreSQL connection string as an environment variable in the Render dashboard
3. Set the `KEEP_ALIVE_INTERVAL_SECS` to 30 (or your preferred interval)
4. Set the port to 3030

The application will automatically detect its Render URL - you don't need to specify it manually.

## Customizing the UI Further

If you want to further customize the UI, you may need to modify:

1. The class/file names in the source code
2. Package dependencies in `package.json`
3. The colors and styling in the UI components

## Security Considerations

- The application doesn't include authentication by default
- Consider placing it behind a reverse proxy with authentication
- Use HTTPS in production environments
- Never commit database credentials to your repository
- Use environment variables for all sensitive configuration 