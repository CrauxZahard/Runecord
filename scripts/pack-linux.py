"""Packs the staged Linux build with correct Unix modes.

Windows' bsdtar has no `--mode` here and would store the binary as 0644, so
unpacking would lose `+x`. This writes the tar.gz directly with tarfile.

    python scripts/pack-linux.py <stage-parent> <out.tar.gz>
"""

import os
import tarfile
import sys

stage, out = sys.argv[1], sys.argv[2]
app = os.path.join(stage, "Runecord")

with tarfile.open(out, "w:gz") as archive:
    info = archive.gettarinfo(app, arcname="Runecord")
    info.mode = 0o755
    archive.addfile(info)
    for base, dirs, files in os.walk(app):
        dirs.sort()
        for name in sorted(dirs):
            path = os.path.join(base, name)
            entry = archive.gettarinfo(path, arcname=os.path.relpath(path, stage))
            entry.mode = 0o755
            archive.addfile(entry)
        for name in sorted(files):
            path = os.path.join(base, name)
            arcname = os.path.relpath(path, stage)
            entry = archive.gettarinfo(path, arcname=arcname)
            executable = name == "Runecord" or name.endswith(".sh")
            entry.mode = 0o755 if executable else 0o644
            with open(path, "rb") as data:
                archive.addfile(entry, data)

print(f"{out}: {os.path.getsize(out) / (1024 * 1024):.1f} MB")
