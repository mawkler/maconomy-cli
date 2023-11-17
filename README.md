# Maconomy CLI

> [!NOTE]
> This is a work in progress, and not ready for use

## Configuration

Configuration is done either in `./config.toml` or by creating a system variable prefixed with `MACONOMY_<value_name>=value` (where `<value_name>` is the name of the value).

```toml
# ./config.toml
maconomy_url = "Base URL"
company = "Company name"

[login]
username = "username"
password = "password"
```
