# Sipp

![cover](https://sipp.so/assets/og.png)

Minimal code sharing made for self hosting

## About

A while back I released an app called Snippets. While it had lots of polish and the stuff influencers say you need to have, it was also bloated, slow, and had too much vendor lock-in. Sipp is the antitode that takes a different approach. There is no Next.js, shadcn/ui components, or even syntax highlighting. It's just text sharing, all powered by Bun and no other dependencies. From the ground up it's designed to be self hosted and used on your own hardware or VPS so you have control of the data. I've also designed this with longevity in mind by using just html css and js. You can edit any part you like, update it, and know that you can always come back and fix something.

The web needs more simplicity, and the ability to build our own tools and solutions that can be picked up and used 5 or 10 years from now. Put down the framework, build simply, and make it open source.

## Quickstart

1. Make sure [Bun](https://bun.com) is installed

```bash
bun --version
```

2. Clone and install types

```bash
git clone https://github.com/stevedylandev/sipp
cd sipp
bun install
```

3. Run the dev server

```bash
bun dev
# 🚀 Server running at http://localhost:3000/
```

## CLI Script

Included with this repos is a bash script `snippet.sh` that you can use to easily create snippets from local files. Follow the instructions below to use it!

1. Update the server URL

At the top of the bash script you should see a constant that you can update to be your deployed or local server URL

```bash
#!/usr/bin/env bash

# Server URL configuration
SERVER_URL="http://localhost:3000"
```

2. Make the script executable

```bash
chmod +x create-snippet.sh
```

3. Move it into a binary path

This will vary from machine or OS, but most people will have a place where downloaded executables are stored and that folder is added to the shell path through a `bashrc` or `zshrc` file. An example might be `~/.local/share`. You can leave off the `.sh` file extension so you don't need to include it while utilizing the script.

```bash
cp snippet.sh ~/.local/share/snippet
```

4. Usage

Simply point provide the path to the file you want to create a snippet for and the script will return the URL for said snippet, as well as copy the URL to your clipboard if supported.

```bash
snippet src/index.ts
```

## Project Structure

```
sipp/
├── src/
│   ├── index.ts          # Main server file with routes and API endpoints
│   ├── db.ts             # SQLite database operations and schema
│   ├── index.html        # Home page with snippet creation form
│   ├── snippet.html      # Snippet viewing page with copy functionality
│   ├── styles.css        # Minimal CSS styling with custom fonts
│   └── assets/
│       ├── site.webmanifest    # Web app manifest
│       └── fonts/              # Custom Commit Mono font files
└── sipp.sqlite          # SQLite database (created automatically)
```

The architecture is intentionally simple:
- **`index.ts`** - Bun server with file-based routing and JSON API
- **`db.ts`** - Direct SQLite operations with no ORM dependencies
- **HTML files** - Plain HTML with inline JavaScript, no build step required
- **`styles.css`** - Single CSS file with custom font loading
- **SQLite database** - File-based storage for maximum portability

## Deployment

Since Sipp is a basic Bun app, all you need is a server enviornment that can install and run the `start` script.

### Self Hosting

If you are running a VPS or your own hardware like a Raspberry Pi, you can use a basic `systemd` service to manage the instance.

1. Clone the repo and install

```bash
git clone https://github.com/stevedylandev/sipp
cd sipp
bun install
```

2. Create a systemd service

The location of where these files are located might depend on your linux distribution, but most commonly they can be found at `/etc/systemd/system`. Create a new file called `sipp.service` and edit it with `nano` or `vim`.

```bash
cd /etc/systemd/service
touch sipp.service
sudo nano sipp.service
```

Paste in the following code:

```bash
[Unit]
# describe the app
Description=Sipp
# start the app after the network is available
After=network.target

[Service]
# usually you'll use 'simple'
# one of https://www.freedesktop.org/software/systemd/man/systemd.service.html#Type=
Type=simple
# which user to use when starting the app
User=YOURUSER
# path to your application's root directory
WorkingDirectory=/home/YOUR_USER/sipp
# the command to start the app
# requires absolute paths
ExecStart=/home/YOUR_USER/.bun/bin/bun start
# restart policy
# one of {no|on-success|on-failure|on-abnormal|on-watchdog|on-abort|always}
Restart=always

[Install]
# start the app automatically
WantedBy=multi-user.target
```

> [!NOTE]
> Make sure you update the `YOUR_USER` with your own user info, and make sure the paths to `bun` and the `sipp` directory are correct!

3. Start up the service

Run the following commands to enable and start the service

```bash
sudo systemctl enable sipp.service
sudo systemctl start sipp
```

Check and make sure it's working

```bash
sudo systemctl status
```

4. Setup a Tunnel (optional)

From here you have a lot of options of how you may want to access the sipp instance. One easy way to start is to use a Cloudflare tunnel and point it to `http://localhost:3000`.


### Docker

1. Clone the repo

```bash
git clone https://github.com/stevedylandev/sipp
cd sipp
```

2. Build and run the Docker image

```bash
docker build -t sipp .
docker run -p 3000:3000 -v $(pwd)/data:/usr/src/app/data sipp
```

Or use `docker-compose`

```bash
docker-compose up -d
```

### Railway

1. Fork the repo from GitHub to your own account

2. Login to [Railway](https://railway.com) and create a new project

3. Select Sipp from your repos

4. Make sure the start command is `bun run start`
