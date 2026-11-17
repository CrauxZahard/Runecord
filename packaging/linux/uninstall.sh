#!/bin/sh
# Removes the Runecord program files installed by install.sh.
#
# Your data in $XDG_DATA_HOME/runecord is kept. Pass --purge to delete it too.

set -eu

prefix=${PREFIX:-"$HOME/.local"}
data_home=${XDG_DATA_HOME:-"$HOME/.local/share"}
purge=0
if [ "${1:-}" = "--purge" ]; then
    purge=1
fi

rm -f "$prefix/bin/runecord"
rm -f "$data_home/applications/runecord.desktop"
rm -f "$data_home/icons/hicolor/256x256/apps/runecord.png"

command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$data_home/applications" >/dev/null 2>&1 || true

if [ "$purge" -eq 1 ]; then
    rm -rf "$data_home/runecord"
    echo "Removed Runecord and its data."
else
    echo "Removed Runecord. Your data is still in $data_home/runecord"
    echo "Re-run with --purge to delete it as well."
fi
