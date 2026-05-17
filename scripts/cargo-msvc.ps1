$ErrorActionPreference = 'Stop'
# Import VS environment variables
$vcvars = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat'
$envOutput = & cmd /c "`"$vcvars`" >nul 2>&1 && set" 2>&1
foreach ($line in $envOutput) {
    if ($line -match '^([^=]+)=(.*)$') {
        $name = $matches[1]
        $value = $matches[2]
        [Environment]::SetEnvironmentVariable($name, $value, 'Process')
    }
}
# Now run cargo with the VS env loaded
& 'C:\Users\WeboTech-Studio\.cargo\bin\cargo.exe' @args
