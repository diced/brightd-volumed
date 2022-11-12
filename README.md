# brightd and volumed

Notification services for controlling volume (pulseaudio/pipewire) and brightness on Linux.

## Pre-requisites
* [brightnessctl](https://github.com/Hummer12007/brightnessctl)
* pactl (part of `libpulse`, you probably have this already)

## Screenshots
![](https://i.imgur.com/nq7LfP4.png)
![](https://i.imgur.com/nXC2GU8.png)

## Installation

```bash
git clone https://github.com/diced/brightd-volumed
cd brightd-volumed
cargo install --path .
```

## Usage
`brightd` and/or `volumed` must be running for `brightctl`/`volumectl` to work.

Once `brightd` is running, you can increase/decrease screen brightness by running `brightctl inc`/`brightctl dec`.

Once `volumed` is running, you can increase/decrease volume by running `volumectl inc`/`volumectl dec`.
You can toggle mute by running `volumectl toggle-mute`.

When those commands are ran, a box in the center of your screen will appear (see screenshots) with a progress bar and specific icons.
This takes advantage of GTK3, so it will match your GTK3 theme.

### volumectl
```
Usage: volumectl <COMMAND>

Commands:
  increase     Increase volume by [step]
  decrease     Decrease volume by [step]
  toggle-mute  Toggle mute
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```

### brightctl
```
Usage: brightctl <COMMAND>

Commands:
  increase  Increase brightness by [step]
  decrease  Decrease brightness by [step]
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```