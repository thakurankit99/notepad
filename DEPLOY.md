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

## PostgreSQL Setup

The application uses PostgreSQL for document persistence. You'll need to provide a valid PostgreSQL connection string in the format:

```
postgres://username:password@hostname:port/database?sslmode=require
```

**IMPORTANT:** Never store database credentials directly in your codebase or configuration files that will be committed to version control. Always use environment variables or secret management tools for sensitive information.

## Render Deployment

When deploying to Render:

1. Use the `custom.Dockerfile` directly
2. Add your PostgreSQL connection string as an environment variable in the Render dashboard
3. Set the port to 3030

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