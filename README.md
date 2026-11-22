# COSMIC Moon Phase & Sun Times Applet

A lightweight, 100% offline panel applet for the [COSMIC Desktop Environment](https://github.com) that displays the current lunar phase, illumination, upcoming moon events, and local moon/sun rise and set times.

![COSMIC Desktop](https://shields.io)
![Rust](https://shields.io)
![Offline](https://shields.io)

## Features

- **Live Panel Icon:** Displays the current day of the lunar cycle (`1–29`) directly in your COSMIC panel.
- **High-Precision Ephemeris Math:** Built-in pure-Rust implementation of Jean Meeus's *Astronomical Algorithms*—no external web APIs or internet connection required.
- **Moon Phase & Illumination:** Shows current phase name, illumination percentage, lunar age, and countdown to the next Full Moon or New Moon.
- **Today & Tomorrow Horizon Times:** Calculates local moonrise, moonset, sunrise, and sunset times for both today and tomorrow.
- **Intuitive Distance Metrics:** Displays Earth–Moon and Earth–Sun distances in kilometers as well as how many Moons, Earths, or Suns fit in the gap between them.
- **Automatic or Manual Location:** Automatically detects your OS timezone and coordinates from bundled IANA timezone data (`zone1970.tab` & `iso3166.tab`), or lets you search and select any city worldwide in its own local timezone.
- **Desktop Notifications:** Optional native COSMIC desktop notifications ahead of upcoming New and Full Moons, configurable from `1 hour` up to `14 days` in advance.

## Dependencies

To build from source, you need a standard Rust toolchain (`cargo` / `rustc`) and the `libcosmic` build dependencies for your distribution:

- **Pop!_OS / Ubuntu / Debian:**
  ```bash
  sudo apt install cargo rustc just libxkbcommon-dev libwayland-dev pkg-config
  ```
- **Fedora:**
  ```bash
  sudo dnf install cargo rust just libxkbcommon-devel wayland-devel pkgconf-pkg-config
  ```
- **Arch Linux:**
  ```bash
  sudo pacman -S rust just libxkbcommon wayland pkgconf
  ```

## Installation

Clone the repository and install using [`just`](https://github.com):

```bash
git clone https://github.com
cd cosmic-applet-moon
```

### Per-User Installation (Recommended, no `sudo` required)

Installs the binary to `~/.local/bin`, the `.desktop` entry to `~/.local/share/applications`, and the icon to `~/.local/share/icons/hicolor`:

```bash
just install
```

*(Make sure `~/.local/bin` is in your `$PATH`.)*

### System-Wide Installation

Installs to `/usr/local/bin` and `/usr/local/share`:

```bash
sudo just install-system
```

## Enabling the Applet

1. Open **COSMIC Settings** → **Desktop** → **Panel** (or **Dock**) → **Applets**.
2. Click **Add applet** and select **Moon Phase & Sun Times**.
3. Click the moon icon in your panel to view details, change your city, or configure New/Full Moon notifications.

## Uninstalling

- **Per-user:**
  ```bash
  just uninstall
  ```
- **System-wide:**
  ```bash
  sudo just uninstall-system
  ```

## License

This project is open source—see the `LICENSE` file for details.
