; SysMonitor NSIS installer hooks.
; Auto-close any running instances before extracting files so we never hit
; "Error opening file for writing: LibreHardwareMonitorLib.dll" because the
; running app holds the DLL.

!macro NSIS_HOOK_PREINSTALL
  DetailPrint "Closing any running SysMonitor instances..."
  nsExec::ExecToLog 'taskkill /F /T /IM sysmonitor.exe'
  nsExec::ExecToLog 'taskkill /F /T /IM sysmon-sensor.exe'
  nsExec::ExecToLog 'taskkill /F /T /IM sysmon-sensor-x86_64-pc-windows-msvc.exe'
  Sleep 600
!macroend

!macro NSIS_HOOK_POSTINSTALL
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Closing any running SysMonitor instances..."
  nsExec::ExecToLog 'taskkill /F /T /IM sysmonitor.exe'
  nsExec::ExecToLog 'taskkill /F /T /IM sysmon-sensor.exe'
  nsExec::ExecToLog 'taskkill /F /T /IM sysmon-sensor-x86_64-pc-windows-msvc.exe'
  Sleep 600
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
!macroend
