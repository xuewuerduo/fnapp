# OUI 厂商数据库生成脚本
# 从 IEEE 官方 OUI 列表提取 MAC 前缀与厂商名称
# 产物: oui.csv（约 40K 条，1.2MB）
# 使用方法: .\data\generate-oui.ps1
# 输出: data\oui.csv（覆盖）

$url = "https://standards-oui.ieee.org/oui/oui.csv"
$output = Join-Path $PSScriptRoot "oui.csv"

Write-Host "正在下载 IEEE OUI 列表..." -ForegroundColor Cyan
try {
    $ProgressPreference = "SilentlyContinue"
    $raw = [System.Net.WebClient]::new().DownloadString($url)
} catch {
    Write-Host "下载失败: $_" -ForegroundColor Red
    exit 1
}

Write-Host "正在解析..." -ForegroundColor Cyan
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

    $null = $builder.AppendLine("$prefix,$vendor")
    $total++
}

[System.IO.File]::WriteAllText($output, $builder.ToString())
Write-Host "完成！共 $total 条记录，输出到 $output" -ForegroundColor Green
