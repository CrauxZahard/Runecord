; Runecord's per-user Windows installer.
;
; Built with Inno Setup (ISCC.exe). The result is one .exe that installs under
; %LOCALAPPDATA%\Programs\Runecord without an elevation prompt, adds Start-menu
; and optional desktop shortcuts, registers an uninstaller, and keeps the user's
; data in %APPDATA%\Runecord when the app is removed.
;
;   ISCC.exe /DAppVersion=0.1.0 /DSourceDir=... /DOutputDir=... runecord.iss

#ifndef AppVersion
  #define AppVersion "0.1.0"
#endif
#ifndef SourceDir
  #define SourceDir "..\..\dist"
#endif
#ifndef OutputDir
  #define OutputDir "..\..\dist"
#endif

#define AppName "Runecord"
#define AppPublisher "Runecord"
#define AppExe "Runecord.exe"
#define AppId "{{7C1F1D8E-2A7B-4C63-9F2A-6E1B0C9D4A11}"

[Setup]
AppId={#AppId}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher={#AppPublisher}
VersionInfoVersion={#AppVersion}
VersionInfoCompany={#AppPublisher}
VersionInfoDescription={#AppName} installer
VersionInfoProductName={#AppName}
DefaultDirName={localappdata}\Programs\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir={#OutputDir}
OutputBaseFilename=Runecord-Setup-{#AppVersion}
SetupIconFile={#SourceDir}\assets\icon.ico
UninstallDisplayIcon={app}\{#AppExe}
UninstallDisplayName={#AppName}
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
RestartApplications=no
AllowNoIcons=yes
UsePreviousAppDir=yes
LicenseFile={#SourceDir}\LICENSE
InfoBeforeFile=
AppMutex=RunecordAppMutex

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#SourceDir}\Runecord.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\assets\icon.png"; DestDir: "{app}\assets"; Flags: ignoreversion
Source: "{#SourceDir}\README.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExe}"; WorkingDir: "{app}"
Name: "{group}\Uninstall {#AppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExe}"; Description: "{cm:LaunchProgram,{#StringChange(AppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
; Only the installed program files. User data stays in %APPDATA%\Runecord.
Type: filesandordirs; Name: "{app}\assets"
