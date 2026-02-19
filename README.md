# Sipp

![cover](https://sipp.so/assets/og.png)

Minimal code sharing in Rust

## Features

- Single binary for Server, CLI, and TUI
- Web UI to create, copy, and share code snippets with syntax highlighting
- Interactive TUI with authenticated access for snippet management

## Quickstart

**1. Install with Cargo**

Install the binary from Crates.io

```bash
cargo install sipp-so
```

To confirm it was installed correctly run the following

```bash
sipp --help
```

**2. Start Server**

For demo purposes you can run this locally, but ideally this would be run in a deployment server with a proper ENV setup with your admin key.

```bash
sipp server --port 3000
```

**3. Create a Snippet**

You can either open up `http://localhost:3000` and create a snippet in a web browser, or use the TUI. In the same directory, open a new terminal window and use 

```bash
# Path to file
sipp path/to/file.rs

# Or use the interactive tui 
sipp
```

## Server

Sipp includes a built-in web server powered by Axum. Start it with:

```bash
sipp server --port 3000 --host localhost
```

### Environment Variables

| Variable | Description |
|---|---|
| `SIPP_API_KEY` | API key for protecting endpoints |
| `SIPP_AUTH_ENDPOINTS` | Comma-separated list of endpoints requiring auth: `api_list`, `api_create`, `api_get`, `api_delete`, `all`, or `none` (defaults to `api_delete,api_list`) |

The server stores snippets in a local `sipp.sqlite` SQLite database.

### API Endpoints

| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/snippets` | List all snippets |
| `POST` | `/api/snippets` | Create a snippet (`{"name": "...", "content": "..."}`) |
| `GET` | `/api/snippets/{short_id}` | Get a snippet by ID |
| `DELETE` | `/api/snippets/{short_id}` | Delete a snippet by ID |

Authenticated endpoints require an `x-api-key` header.

## Web UI

The server serves a web interface at the root URL where you can create, view, and share code snippets with syntax highlighting. Each snippet gets a shareable link at `/s/{short_id}`.

## CLI

Upload a file directly from the command line:

```bash
sipp path/to/file.rs
```

This creates a snippet and prints the shareable link (also copied to clipboard).

## TUI

Launch the interactive terminal UI:

```bash
sipp
```

### Keybindings

| Key | Action |
|---|---|
| `j`/`↓` | Move down / Scroll down |
| `k`/`↑` | Move up / Scroll up |
| `Enter` | Focus content pane |
| `Esc` | Back / Quit |
| `y` | Copy snippet content |
| `Y` | Copy snippet link |
| `o` | Open in browser |
| `d` | Delete snippet |
| `c` | Create snippet |
| `r` | Refresh snippets (remote only) |
| `q` | Quit |
| `?` | Toggle help |

## Configuration

Save your remote URL and API key to `~/.config/sipp/config.toml` so you can access your sipp db anywhere:

```bash
sipp auth
```

You can also pass these as flags or environment variables:

```bash
sipp --remote http://your-server.com --api-key YOUR_KEY
# or
export SIPP_REMOTE_URL=http://your-server.com
export SIPP_API_KEY=YOUR_KEY
```

## Deployment

### Systemd

Create a service file at `/etc/systemd/system/sipp.service`:

```ini
[Unit]
Description=Sipp snippet server
After=network.target

[Service]
ExecStart=/usr/local/bin/sipp server --port 3000 --host 0.0.0.0
Environment=SIPP_API_KEY=your-secret-key
WorkingDirectory=/var/lib/sipp
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable --now sipp
```

### Docker

A `Dockerfile` and `docker-compose.yml` are included in the repository.

```bash
# Using Docker Compose (recommended)
SIPP_API_KEY=your-secret-key docker compose up -d

# Or build and run manually
docker build -t sipp .
docker run -p 3000:3000 -e SIPP_API_KEY=your-secret-key -v sipp-data:/data sipp
```

### Railway

1. Connect your repository to [Railway](https://railway.app)
2. Set the environment variables `SIPP_API_KEY` and optionally `SIPP_AUTH_ENDPOINTS`
3. Set the start command: `sipp server --port $PORT --host 0.0.0.0`
