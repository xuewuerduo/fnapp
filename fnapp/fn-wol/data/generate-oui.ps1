$url = "https://standards-oui.ieee.org/oui/oui.csv"
$output = Join-Path $PSScriptRoot "oui.csv"

Write-Host "Downloading IEEE OUI list..." -ForegroundColor Cyan
try {
    $ProgressPreference = "SilentlyContinue"
    $raw = [System.Net.WebClient]::new().DownloadString($url)
} catch {
    Write-Host "Download failed: $_" -ForegroundColor Red
    exit 1
}

Write-Host "Parsing..." -ForegroundColor Cyan
$builder = New-Object System.Text.StringBuilder
$total = 0

foreach ($line in $raw -split "`n") {
    $i1 = $line.IndexOf(',')
    if ($i1 -le 0) { continue }
    $i2 = $line.IndexOf(',', $i1 + 1)
    if ($i2 -le 0) { continue }
    $prefix = $line.Substring($i1 + 1, $i2 - $i1 - 1)
    if ($prefix.Length -ne 6) { continue }

    $vendorStart = $i2 + 1
    if ($line[$vendorStart] -eq '"') {
        $end = $line.IndexOf('"', $vendorStart + 1)
        if ($end -le 0) { continue }
        $vendor = $line.Substring($vendorStart + 1, $end - $vendorStart - 1)
    } else {
        $end = $line.IndexOf(',', $vendorStart)
        $vendor = $line.Substring($vendorStart, $end - $vendorStart)
    }

    if ($vendor -match ',') {
        $vendor = '"' + $vendor + '"'
    }
    $null = $builder.AppendLine("$prefix,$vendor")
    $total++
}

[System.IO.File]::WriteAllText($output, $builder.ToString())
Write-Host "Done! $total records written to $output" -ForegroundColor Green
