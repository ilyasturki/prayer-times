prayers-makkah:
    cargo run -- --latitude 21.42664 --longitude 39.82563 --method MAKKAH --timezone Asia/Riyadh prayers

prayers-cairo:
    cargo run -- --latitude 30.0444 --longitude 31.2357 --method EGYPT --timezone Africa/Cairo prayers

prayers-istanbul:
    cargo run -- --latitude 41.0082 --longitude 28.9784 --method TURKEY --madhab Hanafi --timezone Europe/Istanbul prayers

prayers-medina:
    cargo run -- --latitude 24.4681 --longitude 39.6142 --timezone Asia/Riyadh prayers

prayers-paris:
    cargo run -- --latitude 48.8566 --longitude 2.3522 --method FRANCE --timezone Europe/Paris prayers

build-nix:
    nix build

run-nix:
    nix run

# Usage: just release patch      (or: minor / major / 0.4.1)
release level:
    claude -p "/changelog for a {{ level }} release"
    cargo release {{ level }} --execute
