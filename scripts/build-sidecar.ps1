# Build the C# sidecar (sysmon-sensor.exe) using csc.exe from VS Build Tools.
# Does NOT require .NET SDK 8 — only .NET Framework 4.8 Dev Pack + VS Build Tools.
# Requires LibreHardwareMonitorLib.dll + HidSharp.dll already present in sidecar/lib/

$ErrorActionPreference = 'Stop'

$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$sidecar = Join-Path $root 'sidecar'
$lib = Join-Path $sidecar 'lib'
$outDir = Join-Path $root 'src-tauri\binaries'
$outExe = Join-Path $outDir 'sysmon-sensor-x86_64-pc-windows-msvc.exe'

$csc = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\MSBuild\Current\Bin\Roslyn\csc.exe'
if (-not (Test-Path $csc)) {
    throw "csc.exe not found at $csc. Install VS 2022 Build Tools with C++ workload."
}

$lhm = Join-Path $lib 'LibreHardwareMonitorLib.dll'
$hid = Join-Path $lib 'HidSharp.dll'
foreach ($dll in @($lhm, $hid)) {
    if (-not (Test-Path $dll)) { throw "Missing dependency DLL: $dll" }
}

New-Item -ItemType Directory -Force -Path $outDir | Out-Null

$src = Join-Path $sidecar 'Program.cs'
& $csc /target:exe /platform:x64 /optimize+ /nologo `
    /reference:"$lhm" `
    /reference:"$hid" `
    /out:"$outExe" `
    "$src"
if ($LASTEXITCODE -ne 0) { throw "csc.exe failed with exit code $LASTEXITCODE" }

# Copy dependent DLLs next to the sidecar exe so DLL search finds them at runtime
Copy-Item -Force $lhm (Join-Path $outDir 'LibreHardwareMonitorLib.dll')
Copy-Item -Force $hid (Join-Path $outDir 'HidSharp.dll')

Write-Host "Sidecar built: $outExe"
Write-Host "Dependencies copied: LibreHardwareMonitorLib.dll, HidSharp.dll"
$sz = (Get-Item $outExe).Length
Write-Host "Size: $([math]::Round($sz / 1KB, 1)) KB"
