
# Meet Link Creator

This project is a command-line tool that creates Google Calendar events with Google Meet links. The tool authenticates via OAuth 2.0, creates a calendar event, copies the Meet link to the clipboard, and opens it in the default browser.

## Features

- Creates a Google Calendar event with Google Meet integration.
- Automatically generates a Meet link and copies it to the clipboard.
- Opens the Meet link in your default web browser.
- Deletes the created calendar event afterward.

## Dependencies

- **Rust Crates**
  - `clap` - For command-line argument parsing.
  - `lazy_static` - For managing shared, mutable global state.
  - `reqwest` - For making HTTP requests.
  - `serde` - For serializing and deserializing JSON data.
  - `yup_oauth2` - For OAuth 2.0 authentication.
  - `tokio` - For async runtime.

- **System Requirements**
  - `xclip` - Used for copying the Meet link to the clipboard (Linux).
  - Browser capable of opening URLs automatically.

## Setup

1. Install `xclip`:
   ```bash
   sudo apt install xclip
   ```

2. Download OAuth 2.0 credentials from the [Google Developer Console](https://console.developers.google.com/) and save them as `credentials.json` in your data directory (`~/.meet_data/` by default).

## Usage

```bash
meet_link_creator [OPTIONS]
```

### Options:
- `--data <PATH>`: Specify the data directory for storing credentials and token cache. Defaults to `~/.meet_data/`.
