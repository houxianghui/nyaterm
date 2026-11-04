# 验证 Windows 可执行文件的版本信息是否符合规范
# 用法:
#   pwsh scripts/ci/verify-version.ps1 -ExePath .\dist\myapp.exe -ExpectedCompany rangerhou -ExpectedProduct NyaTerm
# 不传期望值时仅做非空检查；对 DLL 同样适用
# 退出码: 0=PASS, 1=FAIL, 2=文件不存在
param(
    [Parameter(Mandatory = $true)]
    [string]$ExePath,

    [string]$ExpectedCompany,

    [string]$ExpectedProduct
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $ExePath)) {
    Write-Host "FAIL: 文件不存在: $ExePath" -ForegroundColor Red
    exit 2
}

$vi = (Get-Item -LiteralPath $ExePath).VersionInfo

Write-Host "=== $ExePath ==="
$vi | Format-List CompanyName, ProductName, FileDescription, FileVersion, `
    ProductVersion, InternalName, OriginalFilename, LegalCopyright

$failures = @()

if ([string]::IsNullOrWhiteSpace($vi.CompanyName)) {
    $failures += "CompanyName 为空（企业终端管理软件将无法识别软件归属）"
}
elseif ($ExpectedCompany -and $vi.CompanyName -ne $ExpectedCompany) {
    $failures += "CompanyName='$($vi.CompanyName)'，期望 '$ExpectedCompany'"
}

if ([string]::IsNullOrWhiteSpace($vi.ProductName)) {
    $failures += "ProductName 为空"
}
elseif ($ExpectedProduct -and $vi.ProductName -ne $ExpectedProduct) {
    $failures += "ProductName='$($vi.ProductName)'，期望 '$ExpectedProduct'"
}

if ([string]::IsNullOrWhiteSpace($vi.FileDescription)) { $failures += "FileDescription 为空" }
if ([string]::IsNullOrWhiteSpace($vi.FileVersion))      { $failures += "FileVersion 为空" }
if ([string]::IsNullOrWhiteSpace($vi.ProductVersion))   { $failures += "ProductVersion 为空" }

if ($failures.Count -gt 0) {
    $failures | ForEach-Object { Write-Host "FAIL: $_" -ForegroundColor Red }
    exit 1
}

Write-Host "PASS: 版本信息符合规范" -ForegroundColor Green
