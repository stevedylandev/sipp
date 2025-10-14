# Sipp

![cover](https://sipp.so/assets/og.png)

Minimal code sharing made for self hosting

## About

A while back I released an app called Snippets. While it had lots of polish and the stuff influencers say you need to have, it was also bloated, slow, and had too much vendor lock-in. Sipp is the antitode that takes a different approach. There is no Next.js, shadcn/ui components, or even syntax highlighting. It's just text sharing, all powered by Bun and no other dependencies. From the ground up it's designed to be self hosted and used on your own hardware or VPS so you have control of the data. I've also designed this with longevity in mind by using jut html css an js. You can edit any part you like, update it, and know that you can always come back and fix something.

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

// Todo

### Docker

// Todo

### Railway

1. Fork the repo from GitHub to your own account

2. Login to [Railway](https://railway.com) and create a new project

3. Select Sipp from your repos
