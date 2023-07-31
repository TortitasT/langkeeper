## Development

Create database and run migrations (requires diesel_cli).

```bash
diesel setup
```

## Diesel with sqlite install on Windows for development

Install sqlite with chocolatey.

```powershell
choco install sqlite
```

Create `.lib` with the following command on the directory where the `dll` is.

```powershell
cd C:\ProgramData\chocolatey\lib\SQLite\tools
lib /def:sqlite3.def /out:sqlite3.lib
```

Add path to environment variable.

```powershell
$Env:SQLITE3_LIB_DIR = "C:\ProgramData\chocolatey\lib\SQLite\tools"
```

Install diesel as the oficial site says.
