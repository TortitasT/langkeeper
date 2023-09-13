# Langkeeper

Keep track of the time you spend on each language.

[vscode extension](https://github.com/TortitasT/vscode-langkeeper)

[neovim extension](https://github.com/TortitasT/langkeeper.nvim)

![imagen](https://github.com/TortitasT/langkeeper/assets/102045600/8d0b83fb-2f5f-49f3-a73d-4d043b17fab5)

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
cargo install cargo-watch
cargo watch -x "run serve"
```

## Deploy

Clone

```bash
git clone https://github.com/tortitast/langkeeper
```

Build

```bash
cargo build --release
```

Copy service

```bash
cp ./langkeeper.service /etc/systemd/system/
```

Start service

```bash
service langkeeper start
```

## Tests

Run with logs and in single thread so sqlite database works.

```bash
cargo test -- --nocapture --test-threads=1
```
