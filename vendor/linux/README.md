# Vendored Linux link input

`libxkbcommon.so.0.0.0` is the shared library from Ubuntu 18.04's
`libxkbcommon0 0.8.0-1` (`usr/lib/x86_64-linux-gnu/libxkbcommon.so.0.0.0`).

It is here only so a Linux binary can be cross-linked from Windows. zig's lld
on a Windows host cannot open a library it finds through `-L` (it resolves the
name to an absolute Windows path and fails), and the library must be given as a
direct linker input instead; see `scripts/build-linux.ps1` and
`vendor/xkbcommon/build.rs`. The 18.04 build is used because it links against
glibc 2.27, the oldest still in common use.

At runtime the binary loads the distribution's own `libxkbcommon.so.0` through
its SONAME, so this file never ships inside `dist`. Its licence is in
`COPYRIGHT` (MIT, X11-style).
