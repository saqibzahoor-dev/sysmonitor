# sysmon-sensor

C# sidecar process for SysMonitor. Reads CPU/GPU temperatures and load via [LibreHardwareMonitorLib](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor) (MPL-2.0). Emits one JSON line per second to stdout.

Dependent DLLs are embedded into the .exe via [Costura.Fody](https://github.com/Fody/Costura) so we ship a single binary.

## Build

```powershell
dotnet publish -c Release -r win-x64
```

Output: `bin/Release/net48/win-x64/publish/sysmon-sensor.exe` (~3 MB single-file).

Or use the helper:

```powershell
../scripts/build-sidecar.ps1
```

## Output schema

```json
{"ts":1715961600000,"cpu":{"temp_c":58.3,"package_w":52.1},"gpus":[{"name":"AMD Radeon RX 7800 XT","load_pct":12.0,"temp_c":47.0,"vram_used_mb":1024,"vram_total_mb":16384}]}
```

`null` is used when a sensor value is unavailable.
