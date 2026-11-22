name := 'cosmic-applet-moon'
appid := 'io.github.gavril_amarant.CosmicAppletMoon'
icon_src := 'assets/icons/moon-6.png'

# Build release binary
build:
    cargo build --release

# Install per-user into ~/.local (no sudo required)
install: build
    install -Dm0755 target/release/{{name}} ~/.local/bin/{{name}}
    install -Dm0644 assets/{{appid}}.desktop ~/.local/share/applications/{{appid}}.desktop
    install -Dm0644 {{icon_src}} ~/.local/share/icons/hicolor/128x128/apps/{{appid}}.png
    -gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor 2>/dev/null || true

# Install system-wide into /usr/local (for distro packaging / multi-user)
install-system: build
    install -Dm0755 target/release/{{name}} /usr/local/bin/{{name}}
    install -Dm0644 assets/{{appid}}.desktop /usr/local/share/applications/{{appid}}.desktop
    install -Dm0644 {{icon_src}} /usr/local/share/icons/hicolor/128x128/apps/{{appid}}.png
    -gtk-update-icon-cache -f -t /usr/local/share/icons/hicolor 2>/dev/null || true

# Uninstall per-user files
uninstall:
    rm -f ~/.local/bin/{{name}}
    rm -f ~/.local/share/applications/{{appid}}.desktop
    rm -f ~/.local/share/icons/hicolor/128x128/apps/{{appid}}.png

# Uninstall system-wide files
uninstall-system:
    rm -f /usr/local/bin/{{name}}
    rm -f /usr/local/share/applications/{{appid}}.desktop
    rm -f /usr/local/share/icons/hicolor/128x128/apps/{{appid}}.png
