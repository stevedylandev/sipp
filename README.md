# Sipp

![cover](https://files.stevedylan.dev/sipp-rust.png)

Minimal code sharing

## Features

- Single binary for web server and TUI
- Create snippets and share on the web
- Interactive TUI with authenticated access for snippet management
- Minimal, fast, and low memory consumption

## Demo

Try it out at [sipp.so](https://sipp.so) or install and use the TUI

```bash
sipp -r https://sipp.so
```

## Quickstart

**1. Installo**

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

## Install

Sipp can be installed several ways

### Cargo
Install with Cargo directly 

```bash
cargo install sipp-so
```

### GitHub Releases
Visit the [releases](https://github.com/stevedylandev/sipp/releases) page and download one of the prebuilt binaries

## Usage

### CLI

```
sipp [OPTIONS] [FILE] [COMMAND]
```

#### Commands

| Command | Description |
|---|---|
| `server` | Start the web server |
| `tui` | Launch the interactive TUI |
| `auth` | Save remote URL and API key to config file |

#### Arguments

| Argument | Description |
|---|---|
| `[FILE]` | File path to create a snippet from |

#### Options

| Option | Description |
|---|---|
| `-r, --remote <URL>` | Remote server URL (e.g. `http://localhost:3000`) (env: `SIPP_REMOTE_URL`) |
| `-k, --api-key <KEY>` | API key for authenticated operations (env: `SIPP_API_KEY`) |

### Server

Sipp includes a built-in web server powered by Axum. Start it with:

```bash
sipp server --port 3000 --host localhost
```

#### Environment Variables

| Variable | Description |
|---|---|
| `SIPP_API_KEY` | API key for protecting endpoints |
| `SIPP_AUTH_ENDPOINTS` | Comma-separated list of endpoints requiring auth: `api_list`, `api_create`, `api_get`, `api_delete`, `all`, or `none` (defaults to `api_delete,api_list`) |
| `SIPP_DB_PATH` | Custom path for the SQLite database file (defaults to `sipp.sqlite` in the working directory) |

The server stores snippets in a local `sipp.sqlite` SQLite database.

#### API Endpoints

| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/snippets` | List all snippets |
| `POST` | `/api/snippets` | Create a snippet (`{"name": "...", "content": "..."}`) |
| `GET` | `/api/snippets/{short_id}` | Get a snippet by ID |
| `PUT` | `/api/snippets/{short_id}` | Update a snippet (`{"name": "...", "content": "..."}`) |
| `DELETE` | `/api/snippets/{short_id}` | Delete a snippet by ID |

Authenticated endpoints require an `x-api-key` header.

### TUI

The Sipp TUI makes it easy to create, copy, share, and manage your snippets either locally or remotely. Launch it with:

```bash
# Launch TUI (default behavior when no file argument is given)
sipp

# Or explicitly
sipp tui

# With remote options
sipp -r https://sipp.so -k your-api-key
```

#### Local Access

If you are running `sipp` in the same directory as the `sipp.sqlite` file created by the server instance, the TUI will automatically access the datebase locally and you can edit it directly.

#### Remote Access

To access a remote instance of Sipp make sure to do the following:
- Set the `SIPP_API_KEY` variable in your server instance
- Run `sipp auth` to enter in your server instance URL and the API key, which will be stored under `$HOME/.config/sipp`. You can also set these with the ENV variables `SIPP_REMOTE_URL` and `SIPP_API_KEY`

>[!NOTE]
>You can try a limited remote instance without an API key with `sipp -r https://sipp.so`

#### Actions

While inside the TUI the following actions are available

| Key | Action |
|---|---|
| `j`/`↓` | Move down / Scroll down |
| `k`/`↑` | Move up / Scroll up |
| `Enter` | Focus content pane |
| `Esc` | Back / Quit |
| `y` | Copy snippet content |
| `Y` | Copy snippet link |
| `o` | Open in browser |
| `e` | Edit snippet |
| `d` | Delete snippet |
| `c` | Create snippet |
| `/` | Search snippets |
| `r` | Refresh snippets (remote only) |
| `q` | Quit |
| `?` | Toggle help |

## Deployment

Since Sipp is a single binary it can be run in virtually any enviornment.

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
3. Add a [volume](https://docs.railway.com/guides/volumes) to your service and mount it at `/data`
4. Set `SIPP_DB_PATH` to `/data/sipp.sqlite` so the database persists across deploys
