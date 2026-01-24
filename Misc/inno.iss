; =============================================================================
; WinHider Application - Installer Script
; =============================================================================
;
; Filename: installer.iss
; Author: bigwiz
; Description: Inno Setup configuration for generating the WinHider installer.
;              This script packages the 64-bit binaries, handles file copying,
;              creates shortcuts with correct working directories, and manages
;              uninstallation cleanup.
;
; Key Operations:
; - Defines application metadata (Name, Version, Publisher)
; - Detects system architecture (x64/x86) to launch correct executable
; - Installs main executable, payload DLLs, and version files
; - Configures Start Menu and Desktop shortcuts
; - Sets up modern wizard style with custom banner assets
;
; Designed At - Bitmutex Technologies
; =============================================================================


#define MyAppName "WinHider"
#define MyAppVersion "1.0.6"
#define MyAppPublisher "Bitmutex Technologies"
#define MyAppURL "https://github.com/aamitn/winhider"

[Code]

// App Name  
function MyAppExeName(Param: String): String;
begin
  if IsWin64 then 
    Result := 'Winhider.exe'
  else 
    Result := 'Winhider_32bit.exe';
end;


[Setup]
; Wizard Pages
DisableWelcomePage = no
LicenseFile=..\LICENSE
; Wizard Banner Images
WizardSmallImageFile=.\installer_assets\whicon-bitmap.bmp
WizardImageFile=.\installer_assets\banner.bmp
; NOTE: The value of AppId uniquely identifies this application. Do not use the same AppId value in installers for other applications.
; (To generate a new GUID, click Tools | Generate GUID inside the IDE.)
AppId={{4896775D-F364-4AF8-AD6C-946EE5F49D95}
;SignTool=winsdk_signtool
AppName={#MyAppName} 
AppVersion={#MyAppVersion}
;AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DisableProgramGroupPage=yes
; Uncomment the following line to run in non administrative install mode (install for current user only.)
;PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputBaseFilename=WinhiderInstaller
Compression=lzma
SolidCompression=yes
WizardStyle=modern
; Custom install options
ArchitecturesInstallIn64BitMode=x64compatible 
SetupIconFile=whicon.ico
UninstallDisplayIcon={app}\{code:MyAppExeName}
UninstallDisplayName={#MyAppName}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: checkedonce

[Files]   
Source: "..\target\x86_64-pc-windows-msvc\release\*.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\x86_64-pc-windows-msvc\release\winhider_payload.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\appver.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{code:MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{code:MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{code:MyAppExeName}"; \
Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')} Application}"; \
Flags: nowait postinstall skipifsilent unchecked shellexec; \
WorkingDir: "{app}"

[UninstallDelete]
Type: dirifempty; Name: "{app}"