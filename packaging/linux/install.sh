#!/bin/sh
# Installs Runecord for the current user, no root required.
#
#   ./install.sh            install under ~/.local
#   PREFIX=/usr/local ./install.sh
#
# Runecord keeps its data in $XDG_DATA_HOME/runecord (or ~/.local/share/runecord),
# separately from the program files, so reinstalling never touches it.

set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
prefix=${PREFIX:-"$HOME/.local"}
data_home=${XDG_DATA_HOME:-"$HOME/.local/share"}

if [ ! -f "$here/Runecord" ]; then
    echo "install.sh must sit next to the Runecord binary" >&2
    exit 1
fi

bin_dir="$prefix/bin"
app_dir="$data_home/applications"
icon_dir="$data_home/icons/hicolor/256x256/apps"

install -d "$bin_dir" "$app_dir" "$icon_dir"
install -m 755 "$here/Runecord" "$bin_dir/runecord"
install -m 644 "$here/assets/icon.png" "$icon_dir/runecord.png"

# Keep the UI source next to the app data. The AI editor needs it, and the
# data folder is writable without root.
source_dir="$data_home/runecord/aiedit/src"
if [ -d "$here/src" ]; then
    install -d "$data_home/runecord/aiedit"
    rm -rf "$source_dir"
    cp -R "$here/src" "$source_dir"
fi

sed "s|@BIN@|$bin_dir/runecord|g" "$here/runecord.desktop" > "$app_dir/runecord.desktop"
chmod 644 "$app_dir/runecord.desktop"

# Refresh the desktop database when the tools exist; both are best-effort.
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$app_dir" >/dev/null 2>&1 || true
command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -qtf "$data_home/icons/hicolor" >/dev/null 2>&1 || true

echo "Installed Runecord:"
echo "  $bin_dir/runecord"
echo "  $app_dir/runecord.desktop"
echo
echo "Run it from your application menu, or: $bin_dir/runecord"
echo "Data lives in $data_home/runecord"
if [ -d "$here/src" ]; then
    echo "UI source for the AI editor: $source_dir"
fi
echo "Remove the program files with: $here/uninstall.sh"
