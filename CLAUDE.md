# Project Overview

`prayer-times` is a Rust CLI application that provides Islamic prayer times notifications based on geographical location. It calculates prayer times using the algorithm from praytimes.org and can run as a daemon to send desktop notifications.

# Configuration

The application uses a TOML configuration file located at `$XDG_CONFIG_HOME/prayer-times/config.toml`. Key configuration sections:
- `[prayer]` - Calculation method, madhab, time adjustments
- `[notification]` - Notification settings, urgency, intervals

CLI arguments always take precedence over configuration file settings.

# Reference

- Prayer time calculation methods: https://praytimes.org/calculation
