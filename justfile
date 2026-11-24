name := 'cosmic-applet-moon'
appid := 'io.github.gavril_amarant.cosmic-applet-moon'
icon_src := 'assets/icons/moon-6.png'

deps:
    #!/usr/bin/env bash
    set -e
    . /etc/os-release
    for d in $ID $ID_LIKE; do
        case "$d" in
            fedora)   sudo dnf install -y cargo rust libxkbcommon-devel wayland-devel pkgconf-pkg-config; exit 0 ;;
            ubuntu|debian|pop) sudo apt install -y cargo rustc libxkbcommon-dev libwayland-dev pkg-config; exit 0 ;;
            arch)     sudo pacman -S --noconfirm rust libxkbcommon wayland pkgconf; exit 0 ;;
        esac
    done
    echo "Unsupported: $ID" && exit 1
    
build: deps
    cargo build --release

install: build
    install -Dm0755 target/release/{{name}} ~/.local/bin/{{name}}
    install -Dm0644 assets/{{appid}}.desktop ~/.local/share/applications/{{appid}}.desktop
    install -Dm0644 {{icon_src}} ~/.local/share/icons/hicolor/128x128/apps/{{appid}}.png
    sed -i "s|^Exec=.*|Exec=$HOME/.local/bin/{{name}}|" ~/.local/share/applications/{{appid}}.desktop
    -gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor 2>/dev/null || true

uninstall:
    rm -f ~/.local/bin/{{name}}
    rm -f ~/.local/share/applications/{{appid}}.desktop
    rm -f ~/.local/share/icons/hicolor/128x128/apps/{{appid}}.png
    -gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor 2>/dev/null || true
