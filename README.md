## Diesel with sqlite install on Windows for development

Install sqlite with chocolatey.

```powershell
choco install sqlite
```

Create `.lib` with the following command (needs to be run with Visual Studio Development Console for `lib` to be available) on the directory where the `dll` is.

```powershell
cd C:\ProgramData\chocolatey\lib\SQLite\tools
lib /def:sqlite3.def /out:sqlite3.lib
```

Add path to environment variable.

```powershell
$Env:SQLITE3_LIB_DIR = "C:\ProgramData\chocolatey\lib\SQLite\tools"
```

Install diesel_cli with the following command.

```powershell
cargo install diesel_cli --no-default-features --features postgres
```

## Development

Create database and run migrations (requires diesel_cli).

```bash
diesel setup
```

Run with watch.

```bash
cargo watch -x run
```

## Useful commands

Login

```bash
http --session=./session.json post localhost:8000/users/login email=admin@langmer.es password=secret
```

Ping a language

```bash
http --session=./session.json post localhost:8000/languages/ping extension=.c
```

Get user data

```bash
http --session=./session.json get localhost:8000/users/me
```
