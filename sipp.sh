#!/usr/bin/env bash

# Server URL configuration
SERVER_URL="http://localhost:3000"

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo "Error: jq is not installed"
    echo "Please install jq to use this script:"
    echo "  - Arch Linux: sudo pacman -S jq"
    echo "  - Ubuntu/Debian: sudo apt install jq"
    echo "  - macOS: brew install jq"
    exit 1
fi

# Check if file argument is provided
if [ -z "$1" ]; then
    echo "Usage: $0 <file-path>"
    exit 1
fi

FILE_PATH="$1"

# Check if path exists
if [ ! -e "$FILE_PATH" ]; then
    echo "Error: '$FILE_PATH' not found"
    exit 1
fi

# Check if it's a directory
if [ -d "$FILE_PATH" ]; then
    echo "Error: '$FILE_PATH' is a directory, not a file"
    exit 1
fi

# Check if it's a regular file
if [ ! -f "$FILE_PATH" ]; then
    echo "Error: '$FILE_PATH' is not a regular file"
    exit 1
fi

# Read file content
CONTENT=$(cat "$FILE_PATH")

# Extract filename for the name field
FILENAME=$(basename "$FILE_PATH")

# Make API call to create snippet and capture response
RESPONSE=$(curl -s -X POST "$SERVER_URL/api/snippets" \
    -H "Content-Type: application/json" \
    -d "$(jq -n \
        --arg name "$FILENAME" \
        --arg content "$CONTENT" \
        '{name: $name, content: $content}')")

# Extract shortId from response and print link
SHORT_ID=$(echo "$RESPONSE" | jq -r '.shortId')

if [ -n "$SHORT_ID" ] && [ "$SHORT_ID" != "null" ]; then
    LINK="$SERVER_URL/s/$SHORT_ID"
    echo "$LINK"

    # Copy to clipboard if available
    if command -v wl-copy &> /dev/null; then
        echo -n "$LINK" | wl-copy
        echo "(Copied to clipboard)"
    elif command -v xclip &> /dev/null; then
        echo -n "$LINK" | xclip -selection clipboard
        echo "(Copied to clipboard)"
    elif command -v xsel &> /dev/null; then
        echo -n "$LINK" | xsel --clipboard --input
        echo "(Copied to clipboard)"
    elif command -v pbcopy &> /dev/null; then
        echo -n "$LINK" | pbcopy
        echo "(Copied to clipboard)"
    fi
else
    echo "Error: Failed to create snippet"
    echo "$RESPONSE"
    exit 1
fi
