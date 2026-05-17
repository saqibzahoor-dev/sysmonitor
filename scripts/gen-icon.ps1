# Generate a modern terminal-themed icon source PNG (1024×1024).
# Then run `cargo tauri icon` to produce all required icon sizes + .ico.

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$srcPath = Join-Path $root 'src-tauri\icons\source.png'
$size = 1024

$bmp = New-Object System.Drawing.Bitmap $size, $size, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAliasGridFit
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic

# Transparent canvas
$g.Clear([System.Drawing.Color]::Transparent)

# --- Rounded-rectangle background (dark) ---
function New-RoundedPath {
    param([float]$x, [float]$y, [float]$w, [float]$h, [float]$r)
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $path.AddArc($x, $y, $r*2, $r*2, 180, 90)            # TL
    $path.AddArc($x + $w - $r*2, $y, $r*2, $r*2, 270, 90) # TR
    $path.AddArc($x + $w - $r*2, $y + $h - $r*2, $r*2, $r*2, 0, 90) # BR
    $path.AddArc($x, $y + $h - $r*2, $r*2, $r*2, 90, 90)  # BL
    $path.CloseFigure()
    return $path
}

# Outer shadow / glow halo
$haloPath = New-RoundedPath 8 8 ($size - 16) ($size - 16) 180
$haloBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(60, 0, 255, 65))
$g.FillPath($haloBrush, $haloPath)

# Main rounded panel
$panelPath = New-RoundedPath 56 56 ($size - 112) ($size - 112) 156
$panelBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 13, 17, 23))
$g.FillPath($panelBrush, $panelPath)

# Green border
$borderPen = New-Object System.Drawing.Pen ([System.Drawing.Color]::FromArgb(255, 0, 255, 65)), 10
$g.DrawPath($borderPen, $panelPath)

# Cyan inner stroke for depth
$innerPath = New-RoundedPath 76 76 ($size - 152) ($size - 152) 138
$innerPen = New-Object System.Drawing.Pen ([System.Drawing.Color]::FromArgb(80, 0, 212, 255)), 2
$g.DrawPath($innerPen, $innerPath)

# --- "SYS" monogram ---
$fontFamily = $null
foreach ($name in @('JetBrains Mono', 'Cascadia Code', 'Consolas', 'Lucida Console')) {
    try {
        $ff = New-Object System.Drawing.FontFamily $name
        $fontFamily = $ff
        break
    } catch {}
}
if (-not $fontFamily) { $fontFamily = New-Object System.Drawing.FontFamily 'Courier New' }

$font = New-Object System.Drawing.Font $fontFamily, 280, ([System.Drawing.FontStyle]::Bold), ([System.Drawing.GraphicsUnit]::Pixel)
$textBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 0, 255, 65))
$sf = New-Object System.Drawing.StringFormat
$sf.Alignment = [System.Drawing.StringAlignment]::Center
$sf.LineAlignment = [System.Drawing.StringAlignment]::Center
$rect = New-Object System.Drawing.RectangleF 0, ($size * 0.05), $size, $size
$g.DrawString("SYS", $font, $textBrush, $rect, $sf)

# --- Subtle "monitoring bar" accent at bottom ---
$barX = $size * 0.22
$barY = $size * 0.74
$barW = $size * 0.56
$barH = $size * 0.04
# Track
$trackBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(120, 0, 255, 65))
$g.FillRectangle($trackBrush, $barX, $barY, $barW, $barH)
# Filled portion
$fillBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 0, 255, 65))
$g.FillRectangle($fillBrush, $barX, $barY, $barW * 0.62, $barH)

# Save
New-Item -ItemType Directory -Force -Path (Split-Path $srcPath) | Out-Null
$bmp.Save($srcPath, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose()
$bmp.Dispose()

Write-Host "Source icon saved: $srcPath ($size x $size)"
Write-Host "Next: cd src-tauri && cargo tauri icon icons/source.png"
