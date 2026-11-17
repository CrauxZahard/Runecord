Runecord
========

Windows: run Runecord-Setup-0.1.0.exe, or Runecord.exe for a portable copy.
Linux:   unpack Runecord-linux.tar.gz and run ./install.sh.

The app is self-contained: it starts the backend inside the same process, so
there is no launcher, no console window and no second process to manage.

This build is signed with a self-signed certificate, not a purchased one.
Windows SmartScreen may warn on first launch; choose More info → Run anyway.
You can verify the signature is present (and self-signed) with:

  Get-AuthenticodeSignature .\Runecord.exe

Where the data lives
--------------------
Windows: %APPDATA%\Runecord
Linux:   ~/.local/share/runecord

  config.json            server URL
  session-<profile>.txt  saved sign-in token (plaintext)
  cache-<profile>.json   warm-start cache (plaintext)
  server/runecord.db.enc encrypted database
  server/key.bin         encryption key (back this up with the database)
  server/server.log      server log
  server/backups/        daily encrypted snapshots, newest 7
  server/cdn/            uploaded media (encrypted)

Accounts
--------
A release install starts empty: register the first account from the sign-in
screen, and that account owns the server it creates. Demo accounts (nova and
friends, password "runecord") exist only in development builds.

Forgot a password
-----------------
Stop Runecord, then run:

  Runecord.exe --reset-password USER

A message box shows the new random password. On Linux the same flag prints it
to the terminal. Every session for that account is signed out.

The separate backend exposes the same command:

  runecord-server --reset-password USER

Updates
-------
Settings → Application checks GitHub releases and shows a Download link when a
newer version exists. It never installs anything itself: download the new
installer and run it; your data stays where it is.

Hosting the backend on another machine
--------------------------------------
The client already contains the server. To host it separately, build the
standalone backend and start it with:

  runecord-server --host 0.0.0.0 --data <folder>

Point each client at it from Settings → Application. There is no TLS inside
the server: put a reverse proxy in front of anything reachable beyond a
trusted LAN, because tokens, messages and voice are otherwise unencrypted.

Browser origins are denied by default. A web front end must be listed:

  runecord-server --host 0.0.0.0 --allow-origin https://chat.example.com

Registration is open to anyone who can reach the server. Gate it before
exposing the port:

  runecord-server --host 0.0.0.0 --signup code --signup-code "a-long-shared-code"
  runecord-server --host 0.0.0.0 --signup closed

Login and registration allow 5 attempts per minute per client IP. Behind a
reverse proxy, add --trust-proxy so the limiter reads X-Forwarded-For; do not
add it when clients connect directly.

The desktop build's embedded server reads the same gate from the environment:

  set RUNECORD_SIGNUP=closed
  set RUNECORD_SIGNUP=code
  set RUNECORD_SIGNUP_CODE=your-code

Troubleshooting
---------------
- If nothing appears, read server.log in the data folder above.
- If the app connects to a different backend, check config.json; delete it to
  return to http://127.0.0.1:47800.
- RUNECORD_URL=http://127.0.0.1:47800 forces the local endpoint for a session.
- RUNECORD_DATA=<folder> moves all client and server data for a portable run.
- RUNECORD_NO_UPDATE_CHECK=1 disables the release check.
