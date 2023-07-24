test: target/debug/bitwarden-backup test_valid_backup test_invalid_backup

target/debug/bitwarden-backup:
	cargo build

test_valid_backup:
	@echo TEST $@
	@echo
	(sleep 3 && cat tests/bitwarden_export.json > /tmp/gurken) &
	rust-gdb -batch -x bitwarden-backup.gdb --args target/debug/bitwarden-backup -v -v --path /tmp/gurken
	# serde_json still leaks data see https://github.com/serde-rs/json/issues/874
	set -x && test "$$(grep -ca my-secret-key bitwarden-backup.core)" -le 1
	rm -f bitwarden-backup.core

test_invalid_backup:
	@echo TEST $@
	@echo
	(sleep 3 && echo '{"my-secret-key": "my-secret-key"}' > /tmp/gurken) &
	rust-gdb -batch -x bitwarden-backup.gdb --args target/debug/bitwarden-backup -v -v --path /tmp/gurken
	set -x && test "$$(grep -ca my-secret-key bitwarden-backup.core)" -le 0
	rm -f bitwarden-backup.core
