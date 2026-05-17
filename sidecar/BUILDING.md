# Building the sidecar (sysmon-sensor.exe)

The sidecar provides CPU/GPU temperatures via LibreHardwareMonitorLib. Without it, the GPU tab shows "sensor sidecar unavailable" and CPU temp shows `—` — everything else (CPU usage, RAM, disk, network, processes, hardware inventory) works fine.

## Standard path — with .NET SDK 8

```powershell
# Once-only: install .NET SDK 8 (~200 MB)
winget install Microsoft.DotNet.SDK.8

# Build the sidecar (~3 MB single-file via Costura)
./scripts/build-sidecar.ps1
```

The script publishes the C# project and copies the output exe to
`src-tauri/binaries/sysmon-sensor-x86_64-pc-windows-msvc.exe` where Tauri
expects to find it.

After the sidecar is built, rebuild the installer:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/cargo-msvc.ps1 tauri build
```

## Fallback path — no .NET SDK (uses VS Build Tools' MSBuild)

If you can't install .NET SDK 8 but have VS 2022 Build Tools + .NET Framework
4.8 Developer Pack installed:

1. Download `nuget.exe` standalone from <https://dist.nuget.org/win-x86-commandline/latest/nuget.exe>
2. Restore LibreHardwareMonitorLib:
   ```powershell
   nuget.exe install LibreHardwareMonitorLib -Version 0.9.4 -OutputDirectory ./packages
   ```
3. Convert `SysmonSensor.csproj` to classic (non-SDK) format and reference the
   extracted `LibreHardwareMonitorLib.dll` + `HidSharp.dll` manually.
4. Build with MSBuild:
   ```powershell
   "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\MSBuild\Current\Bin\MSBuild.exe" SysmonSensor.csproj /t:Build /p:Configuration=Release
   ```
5. Bundle the extra DLLs via Tauri `bundle.resources` since the classic build
   won't merge them into the exe automatically.

This path is documented as a fallback only — the standard path with .NET SDK 8
is much simpler.

## Verifying the sidecar works

```powershell
src-tauri/binaries/sysmon-sensor-x86_64-pc-windows-msvc.exe
```

Expected: one JSON line per second on stdout containing CPU temp + GPU array.
Press Ctrl-C to stop.
