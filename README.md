# Maconomy CLI

Maconomy command-line interface for interacting with time sheets.

## Features

- Editing time sheet
  - Editing multiple days at once
- Viewing time sheet, both as table and as JSON
- Automatically instantiating new week if it hasn't been created yet
- Automatically creating new lines if the job/task combination isn't in the time sheet
- Submitting time sheet

## Usage

```
> maconomy set 8 --job 'Some Company' --task 'Development' --day friday
> maconomy get
╭────────────────────────────────────────────────────────────────────╮
│ Job name      Task name         Mon  Tue  Wed  Thu  Fri   Sat  Sun │
├────────────────────────────────────────────────────────────────────┤
│ Some Company  Development                           8:00           │
├────────────────────────────────────────────────────────────────────┤
│ Some Company  More development                                     │
╰────────────────────────────────────────────────────────────────────╯
```

Output of `maconomy --help`:

```
Maconomy command-line interface for interacting with time sheets

Usage: maconomy <COMMAND>

Commands:
  get     Get the time sheet for the current week
  set     Set number of hours on day(s) for a given job and task
  clear   Remove hours on day(s) for a given job and task
  submit  Submit time sheet for week
  logout  Log out
  line    Operate on entire lines in the time sheet
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

Examples:
  maconomy get
  maconomy set 8 --job '<job name>' --task '<task name>'
  maconomy set 8 --job '<job name>' --task '<task name>' --day 'mon-wed, fri' --week 46
  maconomy clear --job '<job name>' --task '<task name>' --day tuesday
  maconomy line delete 2
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

Configuration is done in the file `~/.config/maconomy-cli/config.toml`. Here's what the config file should look like. All fields except `cookie_path` are required:

```toml
# ~/.config/maconomy-cli/config.toml
maconomy_url = "<URL to company's Maconomy API>"
company_id = "<company ID>"

[authentication.sso]
login_url = "<URL to your company's SSO login web page for Maconomy>"
cookie_path = "<path to where auth cookie should be stored>" # Optional, defaults to ~/.local/share/maconomy-cli/maconomy_cookie
```

## Known issues

### "Request failed with status 401 Unauthorized"

**Solution:** Log out with `maconomy logout` and then re-run your previous command.

Sometimes maconomy-cli is unable reauthenticate after your session has expired. maconomy-cli currently authenticates you by opening a browser window, letting you sign in with single-sign on, and then stores your session cookie. It seems like it sometimes it fetches the incorrect authentication cookie. I'll try to fix this, but the optimal solution is to switch to a proper [PKCE](https://auth0.com/docs/get-started/authentication-and-authorization-flow/authorization-code-flow-with-pkce) authentication flow, or similar. However, Maconomy seem to be using some custom authentication on top of SSO that I couldn't get working. That's why I went with the jankier web-browser-cookie-snatching solution.

## Development

To compile and run:

```sh
cargo run

# To pass arguments to maconomy-cli you can use `--`:
cargo run -- set --job '<job name>' --task '<task name>' 8

# To get the full debug log printed to stderr
RUST_LOG=debug cargo run -- ...
```

To run tests:

```sh
cargo test
```
