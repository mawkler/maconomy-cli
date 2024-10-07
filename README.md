# Maconomy CLI

Maconomy command-line interface for interacting with time sheets.

> [!NOTE]
> This repo is in an early stage. Expect some minor issues and breaking changes.

## Usage

Output of `maconomy --help`:

```
Maconomy command-line interface for interacting with time sheets

Usage: maconomy <COMMAND>

Commands:
get Get the time sheet for the current week
set Set number of hours on some day for a given job and task
clear Remove hours hours on some day for a given job and task
logout Log out
help Print this message or the help of the given subcommand(s)

Options:
-h, --help Print help
-V, --version Print version

Examples:
maconomy get
maconomy set --job '<job name>' --task '<task name>' --day tuesday 8
maconomy clear --job '<job name>' --task '<task name>'

NOTE: currently you can only interact with the current week. In the future you'll be able to specify any week.
```

You can also run `maconomy get --help`, `maconomy set --help`, etc. to see more info on how to use each command.

## Installation

```sh
git clone https://github.com/mawkler/maconomy-cli
cargo install --path maconomy-cli/
```

[Here's more information](https://doc.rust-lang.org/cargo/getting-started/installation.html) on how to install cargo (Rust's build-tool).

### Dependencies

- [Chromium](https://chromium.woolyss.com/download/) or Google Chrome (used in the current authentication implementation)

### Configuration

Configuration is done in the file `~/.config/maconomy-cli/config.toml`. Here's what the config file should look like (all fields are required):

```toml
# ~/.config/maconomy-cli/config.toml
maconomy_url = "<URL to company's Maconomy API>"
company_id = "<company ID>"

[authentication.sso]
login_url = "<URL to your company's SSO login web page for Maconomy>"
```

## Known issues

### "Request failed with status 401 Unauthorized"

Sometimes maconomy-cli is unable reauthenticate after your session has expired. maconomy-cli currently authenticates you by opening a browser window, letting you sign in with single-sign on, and then stores your session cookie. It seems like it sometimes it fetches the incorrect authentication cookie. I'll try to fix this, but the optimal solution is to switch to a proper [PKCE](https://auth0.com/docs/get-started/authentication-and-authorization-flow/authorization-code-flow-with-pkce) authentication flow, or similar.

## Development

To compile and run:

```sh
cargo run

# To pass arguments to maconomy-cli you can use `--`:
cargo run -- set --job '<job name>' --task '<task name>' 8
```

To run unit tests:

```sh
cargo test
```
