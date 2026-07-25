#!/bin/env nu


cargo build --bin sync --release;

cp ./target/release/sync ~/bin/vault_sync

systemctl --user start vault_sync
