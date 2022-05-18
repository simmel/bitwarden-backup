# bitwarden-backup

A securer and composable tool for doing backups of Bitwarden.

Uses named pipes (UNIX-like OSes) or as a last resort filesystem notification
(Windows) to securely handle Bitwarded unencrypted JSON backup from the
Bitwarden Web Vault via your webbrowser or via the Bitwarden Desktop application.

## Usage

### UNIX-like
```shell
$ bitwarden-backup --path ~/Downloads/bitwarden_export.json | \
  gpg -eac --passphrase-fd 3 -o bitwarden_export.json.asc \
    3< <(pass bitwarden/backup) && \
  scp bitwarden_export.json.asc backup-server.example.com:/backup && \
  curl -sSf https://some-service/my-uuid/completed
```

then save the unencrypted JSON backup (.json) of your Vault into the file
`~/Downloads/bitwarden_export.json`.

### Windows
```shell
$ bitwarden-backup.exe --path $HOME\Downloads\bitwarden_backup\ | \
  gpg -eac --passphrase-fd 3 -o bitwarden_export.json.asc \
    3< <(pass bitwarden/backup) && \
  scp bitwarden_export.json.asc backup-server.example.com:/backup && \
  curl -sSf https://some-service/my-uuid/completed
```

then save the unencrypted JSON backup (.json) of your Vault into the folder
`$HOME\Downloads\bitwarden_backup\`.

## Securer?

Nothing is secure.

But we can at least try our best:
* Use named pipe so the unencrypted passwords never hit disk and only touch
  memory (only on UNIX-like).
* If we need to hit disk overwrite the file (yeah, we know it probably won't
  help with SSD/NVMEs) (only on Windows)
* Overwrite memory used after we're done with it
* Use a safe(r) programming language (Rust Evangelism Strikeforce assemble!)
* We know gpg-agent stores the passwords in memory for a while but that's not
  our problem ; P
