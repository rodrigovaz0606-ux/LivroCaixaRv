#define MyAppName "Livro Caixa RV"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "Livro Caixa RV"
#define MyAppExeName "livro-caixa-rv.exe"

[Setup]
AppId={{E4E9853C-F97F-4D46-8AA1-8F30D514BA54}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={localappdata}\Programs\Livro Caixa RV
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
OutputDir=output
OutputBaseFilename=Instalador-Livro-Caixa-RV
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
SetupIconFile=..\assets\livro-caixa-rv.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
CloseApplications=yes
RestartApplications=no

[Languages]
Name: "brazilianportuguese"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"

[Files]
Source: "..\target\release\livro-caixa-rv.exe"; DestDir: "{app}"; Flags: ignoreversion

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "LivroCaixaRV"; ValueData: """{app}\{#MyAppExeName}"""; Flags: uninsdeletevalue

[Icons]
Name: "{autodesktop}\Livro Caixa RV"; Filename: "http://127.0.0.1:3000"; IconFilename: "{app}\{#MyAppExeName}"
Name: "{group}\Livro Caixa RV"; Filename: "http://127.0.0.1:3000"; IconFilename: "{app}\{#MyAppExeName}"
Name: "{group}\Desinstalar Livro Caixa RV"; Filename: "{uninstallexe}"

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Iniciar o Livro Caixa RV em segundo plano"; Flags: nowait postinstall skipifsilent
Filename: "http://127.0.0.1:3000"; Description: "Abrir o Livro Caixa RV"; Flags: shellexec postinstall skipifsilent unchecked
