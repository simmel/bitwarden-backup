# bitwarden-backup

A securer and composable tool for doing backups of Bitwarden.

Uses named pipes (UNIX-like OSes) or as a last resort filesystem notification
(Windows) to securely handle Bitwarded unencrypted JSON backup from the
Bitwarden Web Vault via your webbrowser or via the Bitwarden Desktop application.

## Usage

```shell
$ bitwarden-backup --path ~/Downloads/bitwarden_export.json | \
  gpg -eac --output bitwarden_export.json.asc && \
  scp bitwarden_export.json.asc backup-server.example.com:/backup && \
  curl -sSf https://some-service/my-uuid/completed
```

then save the unencrypted JSON backup (.json) of your Vault into the file
`~/Downloads/bitwarden_export.json`.
