$ErrorActionPreference = "Stop"
Push-Location (Join-Path $PSScriptRoot "..\sidecar")
try {
    dotnet publish -c Release -r win-x64
} finally {
    Pop-Location
}
$src = Join-Path $PSScriptRoot "..\sidecar\bin\Release\net48\win-x64\publish\sysmon-sensor.exe"
$dst = Join-Path $PSScriptRoot "..\src-tauri\binaries\sysmon-sensor-x86_64-pc-windows-msvc.exe"
New-Item -ItemType Directory -Force -Path (Join-Path $PSScriptRoot "..\src-tauri\binaries") | Out-Null
Copy-Item -Force $src $dst
Write-Host "Sidecar copied to $dst"
