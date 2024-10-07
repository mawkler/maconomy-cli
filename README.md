# Maconomy CLI

Maconomy command-line interface for interacting with time sheets.

> [!NOTE]
> This repo is in an early stage. Expect some minor issues and breaking changes.

## Installation

```sh
git clone https://github.com/mawkler/maconomy-cli
cargo install --path maconomy-cli/
```

### Configuration

Configuration is done either in `~/.config/maconomy-cli/config.toml` or by creating a system variable prefixed with `MACONOMY_<value_name>=value` (where `<value_name>` is the name of the value). Here's what the config file should look like (all fields are required):

```toml
# ~/.config/maconomy-cli/config.toml
maconomy_url = "<URL to company's Maconomy API>"
company_id = "<company ID>"

[authentication.sso]
login_url = "<URL to your company's SSO login web page for Maconomy>"
```

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

## Known issues

### "Request failed with status 401 Unauthorized"

Sometimes maconomy-cli is unable reauthenticate after your session has expired. maconomy-cli currently authenticates you by opening a browser window, letting you sign in with single-sign on, and then stores your session cookie. It seems like it sometimes it fetches the incorrect authentication cookie. I'll try to fix this, but the optimal solution is to switch to a proper [PKCE](https://auth0.com/docs/get-started/authentication-and-authorization-flow/authorization-code-flow-with-pkce) authentication flow, or similar.
