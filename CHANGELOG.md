# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- systemd user unit (`contrib/prayer-times.service`) shipped via the AUR packages and the Nix flake's `postInstall`.
- NixOS module exposed as `nixosModules.default` with a `services.prayer-times.enable` option.

### Changed

- Daemon output now flows through the `log` crate; per-tick lines are demoted to `debug`. Set `RUST_LOG=debug` to restore the previous verbosity.
- Auto-detected location is logged once at `info` level instead of being printed only during the daemon command.

## [0.4.2] - 2026-04-19

### Added

- `--no-geolocation` flag (and matching config option) to disable the IP-based location fallback.

### Fixed

- Prayer calculation no longer panics at extreme latitudes where the sun does not reach the required altitude; the affected event falls back to midnight instead of aborting.
- Latitude and longitude are now validated on startup, with a clear error message when values are out of range or non-finite.
- Config load errors no longer panic when the underlying error has no source.

## [0.4.1] - 2026-04-17

### Added

- Re-enabled the `dry-run` command.
- Partial config files now load cleanly, with missing fields falling back to defaults.

### Changed

- Breaking: upgraded `confy`, `strum`, `strum_macros`, and `toml` to new major versions.
- Upgraded `chrono`, `clap`, `notify-rust`, `serde`, and `time` to the latest compatible versions.
- Retired the manual PKGBUILD in favour of documented install channels (AUR, crates.io, Nix).
- Removed the `justfile` and its `run` recipe.

### Fixed

- Invalid notification icons are skipped instead of aborting the notification.
- Timezone offset now uses the prayer date rather than the current date.

## [0.4.0] - 2025-10-15

### Added

- Timezone support via `--timezone` / config with fallback to system timezone.
- Date argument for the `prayers` command (list prayers for any day).
- More calculation methods; expanded and refactored method catalog.
- `run` recipe in the justfile for quick cargo execution.

### Changed

- Breaking: API reshuffle across several commands; removed the `dry-run` command.
- Breaking: core prayer calculations now account for the configured timezone.
- Refactored calculation module into shared mathematical utilities.
- Updated prayer coordinates for Makkah and improved degree normalization.
- Migrated development environment from devenv to a Nix shell (flake.nix).

### Fixed

- Correct fallback to system timezone when none is specified.
- Julian day calculation.
- Dhuhr prayer offset and Asr calculation in the southern hemisphere.
- Asr calculation bug and assorted typos.

## [0.3.1] - 2024-04-02

### Added

- Better error logging for location lookup failures.

### Fixed

- PKGBUILD error when generating shell completions without a configured location.

## [0.3.0] - 2024-03-31

### Added

- Shell tab completion generation (`generate-shell`).
- Custom icon path for notifications (`--icon`).
- Interval option for checking new prayers.
- `methods` and `madhab` listing commands.
- TOML configuration file and CLI getter for config path.
- IP-based geolocation when coordinates are not provided.

### Changed

- Daemon now displays the automatically discovered location.
- Technical rework: file management, command-line getters, configuration plumbing.

### Fixed

- Clippy warnings across the codebase.

## [0.2.0] - 2024-02-18

### Added

- Notification urgency as a CLI argument.
- `.SRCINFO` for AUR.

### Fixed

- Icon in release builds.
- Double-notification bug and missing notifications after system suspend.

## [0.1.0] - 2024-01-20

### Added

- Initial release: prayer-time calculation (Fajr, Dhuhr, Asr, Maghrib, Isha)
  with desktop notifications, config-driven method/madhab selection, and
  per-prayer time modifications.
- PKGBUILD for Arch Linux.
- README and project metadata.
