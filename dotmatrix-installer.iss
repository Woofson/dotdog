; dotmatrix Inno Setup Installer Script
; https://jrsoftware.org/isinfo.php
; Requires Inno Setup 6.x

#define AppName "dotmatrix"
#define AppVersion "0.4.2"
#define AppPublisher "Woofson"
#define AppURL "https://github.com/Woofson/dotmatrix"
#define AppCliExeName "dmxcli.exe"
#define AppTuiExeName "dmxtui.exe"

[Setup]
AppId={{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} v{#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppURL}
AppSupportURL={#AppURL}/issues
AppUpdatesURL={#AppURL}/releases

; Default install to Program Files
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}

DisableProgramGroupPage=no

; Output
OutputDir=release
OutputBaseFilename=dotmatrix-{#AppVersion}-setup-windows-x86_64

; Compression
Compression=lzma2/ultra64
SolidCompression=yes
LZMAUseSeparateProcess=yes

; Appearance
WizardStyle=modern
SetupIconFile=assets\dotmatrix-icon.ico

; Minimum Windows version: Windows 10
MinVersion=10.0

; Run as admin so we can write to Program Files and system PATH
PrivilegesRequired=admin

; Uninstaller
UninstallDisplayName={#AppName}
UninstallDisplayIcon={app}\dotmatrix-icon.ico

; Code signing - uncomment and fill in if you have a certificate
; SignTool=signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 /f "path\to\cert.pfx" /p "password" $f
; SignedUninstaller=yes


[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"


[Tasks]
; Offer to add to PATH - checked by default, this is the most important option for a CLI tool
Name: "addtopath"; Description: "Add {#AppName} to your PATH (recommended - lets you run 'dotmatrix' from any terminal)"
Name: "desktopicon"; Description: "Create desktop shortcut"; GroupDescription: "Shortcuts:"


[Files]
Source: "target\release\{#AppCliExeName}"; \
  DestDir: "{app}"; \
  Flags: ignoreversion

Source: "target\release\{#AppTuiExeName}"; \
  DestDir: "{app}"; \
  Flags: ignoreversion

; Example config
Source: "example-config.toml"; \
  DestDir: "{app}"; \
  Flags: ignoreversion

; Icon
Source: "assets\dotmatrix-icon.ico"; \
  DestDir: "{app}"; \
  Flags: ignoreversion

; Docs
Source: "README.md"; \
  DestDir: "{app}"; \
  Flags: ignoreversion

Source: "CHANGELOG.md"; \
  DestDir: "{app}"; \
  Flags: ignoreversion

Source: "LICENSE"; \
  DestDir: "{app}"; \
  Flags: ignoreversion


[Icons]
Name: "{group}\Dot Matrix (TUI)"; \
  Filename: "{app}\{#AppTuiExeName}"; \
  IconFilename: "{app}\dotmatrix-icon.ico"; \
  Comment: "Terminal UI"

Name: "{commondesktop}\Dot Matrix (TUI)"; \
  Filename: "{app}\{#AppTuiExeName}"; \
  IconFilename: "{app}\dotmatrix-icon.ico"; \
  Tasks: desktopicon

Name: "{group}\Dot Matrix CLI (help)"; \
  Filename: "{cmd}"; \
  Parameters: "/k ""{app}\{#AppCliExeName}"" --help"; \
  IconFilename: "{app}\dotmatrix-icon.ico"; \
  Comment: "Open CLI help in Command Prompt"


[Registry]
; Add install dir to user PATH if the task was selected
; Uses HKCU (current user) so it doesn't require elevated rights for the env var itself
Root: HKCU; \
  Subkey: "Environment"; \
  ValueType: expandsz; \
  ValueName: "Path"; \
  ValueData: "{olddata};{app}"; \
  Check: NeedsAddPath('{app}'); \
  Tasks: addtopath; \
  Flags: preservestringtype


[Code]
// Helper function: check if the path is already in PATH to avoid duplicates
function NeedsAddPath(Param: string): boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(
    HKEY_CURRENT_USER,
    'Environment',
    'Path',
    OrigPath
  ) then begin
    Result := True;
    exit;
  end;
  // Check if already present (case insensitive)
  Result := Pos(';' + Uppercase(Param) + ';', ';' + Uppercase(OrigPath) + ';') = 0;
end;

// On uninstall: remove our entry from PATH
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  OldPath: string;
  NewPath: string;
  SearchStr: string;
  P: integer;
begin
  if CurUninstallStep = usPostUninstall then begin
    if RegQueryStringValue(
      HKEY_CURRENT_USER,
      'Environment',
      'Path',
      OldPath
    ) then begin
      SearchStr := ExpandConstant('{app}');
      NewPath := OldPath;

      // Remove ;path variant
      P := Pos(';' + SearchStr, NewPath);
      if P > 0 then
        Delete(NewPath, P, Length(';' + SearchStr))
      else begin
        // Remove path; variant (if it was at the start)
        P := Pos(SearchStr + ';', NewPath);
        if P > 0 then
          Delete(NewPath, P, Length(SearchStr + ';'));
      end;

      if NewPath <> OldPath then
        RegWriteStringValue(
          HKEY_CURRENT_USER,
          'Environment',
          'Path',
          NewPath
        );
    end;
  end;
end;

// Show a finish message reminding the user to restart their terminal
procedure DeinitializeSetup();
begin
  // Nothing needed here - the wizard's finish page handles messaging
end;


[Messages]
; Customise the finish page to remind users to restart their terminal
FinishedLabel=Setup has finished installing [name] on your computer.%n%nIf you added {#AppName} to your PATH, restart open terminals.%n%nRun [bold]dmxcli --help[/bold] or launch [bold]dmxtui.exe[/bold] from Start Menu shortcuts.
