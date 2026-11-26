<#
.SYNOPSIS
    Reproducible cold-startup and memory footprint benchmark for BitChord PC.
.EXAMPLE
    .\benchmarks\measure.ps1 -ExePath ".\target\release\bitchord.exe"
#>
param(
    [string]$ExePath = "$env:LOCALAPPDATA\Programs\BitChord\bitchord.exe",
    [int]$WarmupSeconds = 10,
    [int]$SampleSeconds = 30
)

if (-not (Test-Path $ExePath)) {
    Write-Error "Executable not found at: $ExePath"
    exit 1
}

Write-Host "Benchmarking: $ExePath" -ForegroundColor Cyan
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$proc = Start-Process -FilePath $ExePath -PassThru

# Wait for the main window handle to appear (first GUI frame / window creation)
while ($proc.MainWindowHandle -eq 0 -and -not $proc.HasExited) {
    Start-Sleep -Milliseconds 10
}
$sw.Stop()
Write-Host ("Window ready in: {0:N0} ms" -f $sw.Elapsed.TotalMilliseconds) -ForegroundColor Green

Write-Host "Waiting ${WarmupSeconds}s for idle settle..."
Start-Sleep -Seconds $WarmupSeconds
$proc.Refresh()
$idleWs = $proc.WorkingSet64 / 1MB
$idlePriv = $proc.PrivateMemorySize64 / 1MB
Write-Host ("Idle RAM ({0}s): Working Set = {1:N1} MB | Private = {2:N1} MB" -f $WarmupSeconds, $idleWs, $idlePriv)

Write-Host "Sampling for ${SampleSeconds}s (start playback manually if measuring active playback)..."
Start-Sleep -Seconds $SampleSeconds
$proc.Refresh()
$activeWs = $proc.WorkingSet64 / 1MB
$activePriv = $proc.PrivateMemorySize64 / 1MB
Write-Host ("Active RAM ({0}s): Working Set = {1:N1} MB | Private = {2:N1} MB" -f ($WarmupSeconds + $SampleSeconds), $activeWs, $activePriv)

if (-not $proc.HasExited) {
    Stop-Process -Id $proc.Id -Force
}
