# Prayer Times

## Overview

`prayer-times` is a program that provides Islamic prayer times notifications based on your geographical location. It calculates prayer times for Fajr, Dhuhr, Asr, Maghrib, and Isha using specified calculation methods and adjustments.

It uses accurate calculation of prayer times based on geographical coordinates based on the algorithm provided by [praytimes.org](http://praytimes.org/).

## Installation

### Arch linux

`prayer-times` is available in the [AUR](https://aur.archlinux.org/packages/prayer-times). You can install it with yay or paru depending on what you have.

```sh
yay -S prayer-times
```

or

```sh
paru -S prayer-times
```

### NixOS / Nix

Add the flake input to your NixOS configuration:

```nix
# flake.nix
{
  inputs.prayer-times.url = "github:Yasso9/prayer-times";

  outputs = { nixpkgs, prayer-times, ... }: {
    # Option 1: Add the package directly
    environment.systemPackages = [ prayer-times.packages.${system}.default ];

    # Option 2: Use the overlay
    nixpkgs.overlays = [ prayer-times.overlays.default ];
    environment.systemPackages = [ pkgs.prayer-times ];
  };
}
```

Or run it directly without installing:

```sh
nix run github:Yasso9/prayer-times
```

### Manual

Clone the repository and build the executable. You should have `cargo` installed:

```sh
git clone https://github.com/Yasso9/prayer-times
cd prayer-times
cargo build --release
```

## Usage

```man
Islamic Prayer Times Information and Notifications

Usage: prayer-times [OPTIONS] [COMMAND]

Commands:
  daemon          Start the process that will send notifications on prayers time [default]
  previous        Get the previous prayer
  current         Get the current prayer
  next            Get the next prayer
  prayers         List all the prayers of a specific date (defaults to current day)
  methods         List all methods available for the calculation of the prayer times
  madhab          List all madhab available for the calculation of the prayer times
  config          Get the path of the toml config file
  generate-shell  Generate shell completions and man pages
  help            Print this message or the help of the given subcommand(s)

Options:
  -l, --latitude <LATITUDE>            Latitude. Defaults to the current location
  -L, --longitude <LONGITUDE>          Longitude. Defaults to the current location
  -t, --timezone <TIMEZONE>            Timezone for prayer times (e.g., "America/New_York", "Etc/GMT", "UTC") [default: system timezone]
  -m, --method <METHOD>                Calculation Method to use
  -M, --madhab <MADHAB>                Madhab to use
      --fajr-mod <FAJR_MOD>            Minutes to add or remove to the Fajr time
      --dhuhr-mod <DHUHR_MOD>          Minutes to add or remove to the Dhuhr time
      --asr-mod <ASR_MOD>              Minutes to add or remove to the Asr time
      --maghrib-mod <MAGHRIB_MOD>      Minutes to add or remove to the Maghrib time
      --isha-mod <ISHA_MOD>            Minutes to add or remove to the Isha time
      --notify-before <NOTIFY_BEFORE>  Show notification 10 minutes before prayer time [default: false] [possible values: true, false]
      --icon <ICON>                    Custom icon path for notifications
      --urgency <URGENCY>              Notification urgency
  -h, --help                           Print help
  -V, --version                        Print version
```

You can also configure the program from a config file located at `$XDG_CONFIG_HOME/prayer-times/config.toml` (usually `~/.config/prayer-times/config.toml`). Here is the default config :

```toml
[prayer]
method = "MWL"
madhab = "Shafi"
fajr_mod = 0
dhuhr_mod = 0
asr_mod = 0
maghrib_mod = 0
isha_mod = 0

[notification]
notify_before = false
urgency = "Critical"
interval = 20
```

If you specify CLI arguments, they take precedence over what you have in your config. If you don't specify any latitude and longitude, they will be inferred from your IP address. IP-based location is not very accurate, so specifying your own latitude and longitude is recommended for more accurate prayer times. To set location in the config file, use the `location` section with `lat` and `lon`. CLI flags use `--latitude` and `--longitude`.

## Examples

`prayer-times next`
```
Adhan Dhuhr in 01:13
```

`prayer-times prayers`
```
Fajr at 07:03:06
Sunrise at 08:11:30
Dhuhr at 13:36:18
Asr at 16:28:00
Sunset at 19:01:05
Maghrib at 19:01:05
Isha at 20:09:29
Midnight at 01:36:18
```

`prayer-times methods`
```
Muslim World League : [ fajr: 18°, isha: 17° ]
Islamic Society of North America (ISNA) : [ fajr: 15°, isha: 15° ]
Egyptian General Authority of Survey : [ fajr: 19.5°, isha: 17.5° ]
Umm Al-Qura University, Makkah : [ fajr: 18.5°, isha: 90 min ]
University of Islamic Sciences, Karachi : [ fajr: 18°, isha: 18° ]
Institute of Geophysics, University of Tehran : [ fajr: 17.7°, isha: 14° ]
Shia Ithna-Ashari, Leva Institute, Qum : [ fajr: 16°, isha: 14° ]
Gulf Region : [ fajr: 19.5°, isha: 90 min ]
Kuwait : [ fajr: 18°, isha: 17.5° ]
Qatar : [ fajr: 18°, isha: 90 min ]
Majlis Ugama Islam Singapura, Singapore : [ fajr: 20°, isha: 18° ]
Union Organization Islamic de France : [ fajr: 12°, isha: 12° ]
Diyanet İşleri Başkanlığı, Turkey : [ fajr: 18°, isha: 17° ]
Spiritual Administration of Muslims of Russia : [ fajr: 16°, isha: 15° ]
Dubai : [ fajr: 18.2°, isha: 18.2° ]
Jabatan Kemajuan Islam Malaysia (JAKIM) : [ fajr: 20°, isha: 18° ]
Tunisia : [ fajr: 18°, isha: 18° ]
Algeria : [ fajr: 18°, isha: 17° ]
Kementerian Agama Republik Indonesia : [ fajr: 20°, isha: 18° ]
Morocco : [ fajr: 19°, isha: 17° ]
Comunidade Islamica de Lisboa : [ fajr: 18°, isha: 77 min ]
Ministry of Awqaf, Islamic Affairs and Holy Places, Jordan : [ fajr: 18°, isha: 18° ]
```

`prayer-times madhab`
```
Shafi
Hanafi
```


## License

This project is licensed under the [MIT License](LICENSE). Feel free to use and contribute to this open-source project.
