Runecord
========

Windows: Run Runecord-Setup-0.1.0.exe. Use Runecord.exe for a portable copy.
Linux:   Unpack Runecord-linux.tar.gz. Run ./install.sh.

The app starts its own backend. No launcher or second process is needed.

The Windows build uses a self-signed certificate. SmartScreen may warn.
Choose More info, then Run anyway. Check the signature with:

  Get-AuthenticodeSignature .\Runecord.exe

Data location
-------------
Windows: %APPDATA%\Runecord        Linux: ~/.local/share/runecord

  config.json             Server URL.
  session-<profile>.txt   Saved token. Keep this file private.
  cache-<profile>.json    Warm-start cache.
  server\runecord.db.enc  Encrypted database.
  server\key.bin          Encryption key. Back it up with the database.
  server\server.log       Server log.
  server\backups\         Daily snapshots. The newest 7 remain.
  server\cdn\             Encrypted uploads.

Accounts
--------
A release install starts empty. Register the first account. That account owns its server.
Demo accounts exist only in development builds. Password: runecord.

Reset a password
----------------
Stop Runecord. Then run:

  Runecord.exe --reset-password USER

A message box shows the new password. All sessions for that account end.

Updates
-------
Settings > Application checks for a new release. The app never installs it.
Download the new installer and run it. Your data stays in place.

Separate backend
----------------
Start the backend on another machine:

  runecord-server --host 0.0.0.0 --data <folder>

Point each client at it in Settings > Application. The server has no TLS.
Put a reverse proxy in front of any public port.

Gate signup before you expose the port:

  runecord-server --host 0.0.0.0 --signup code --signup-code CODE
  runecord-server --host 0.0.0.0 --signup closed

Login allows 5 attempts per minute per IP. Behind a proxy, add --trust-proxy.
Do not add --trust-proxy for direct connections.

Troubleshooting
---------------
- Read server.log if the window stays empty.
- Check config.json if the app uses the wrong server. Delete it to reset the URL.
- RUNECORD_URL forces one endpoint for a session.
- RUNECORD_DATA moves all data for a portable run.
- RUNECORD_NO_UPDATE_CHECK=1 disables the release check.
