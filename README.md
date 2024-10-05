# Maconomy CLI

> [!NOTE]
> This is a work in progress, and not ready for use

## Configuration

Configuration is done either in `~/.config/maconomy-cli/config.toml` or by creating a system variable prefixed with `MACONOMY_<value_name>=value` (where `<value_name>` is the name of the value). Here's what the config file should look like (all fields are required):

```toml
# ~/.config/maconomy-cli/config.toml
maconomy_url = "<URL to company's Maconomy API>"
company_id = "<company ID>"

[authentication.sso]
login_url = "<URL to your company's SSO login web page for Maconomy>"
```
