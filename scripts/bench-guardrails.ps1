$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot

function Resolve-ClrinfBinary {
    $release = Join-Path $RepoRoot "tools\clrinf-codegen\target\release\clrinf-codegen.exe"
    $debug = Join-Path $RepoRoot "tools\clrinf-codegen\target\debug\clrinf-codegen.exe"
    $existing = @(@($release, $debug) | Where-Object { Test-Path $_ } | Sort-Object { (Get-Item $_).LastWriteTimeUtc } -Descending)
    if ($existing.Count -gt 0) {
        return $existing[0]
    }
    Write-Host "Building clrinf-codegen..." -ForegroundColor Yellow
    cargo build --manifest-path (Join-Path $RepoRoot "tools\clrinf-codegen\Cargo.toml")
    if ($LASTEXITCODE -ne 0) { throw "Failed to build clrinf-codegen" }
    return $debug
}

$Bin = Resolve-ClrinfBinary
$BenchDir = Join-Path $RepoRoot "examples\guardrails-benchmark"
$TemplatesDir = Join-Path $RepoRoot "tools\clrinf-codegen\templates"

Write-Host "=== Suite 1: Polyglot Guardrails & Event Context Slicing ===" -ForegroundColor Cyan

# 1. Clean baseline verify
Push-Location $BenchDir
try {
    & $Bin verify --json | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "Expected clean baseline in $BenchDir to pass clrinf verify"
    }
    Write-Host "[PASS] Clean baseline passed clrinf verify (0 errors)" -ForegroundColor Green
} finally {
    Pop-Location
}

# 2. Deterministic context-cost measurement
Push-Location $BenchDir
try {
    $benchJsonRaw = & $Bin bench context orders.placed.v1 `
        --schema-dir "contracts\schemas" `
        --src "services" `
        --json
    if ($LASTEXITCODE -ne 0) { throw "clrinf bench context failed" }
} finally {
    Pop-Location
}
$bench = $benchJsonRaw | ConvertFrom-Json

Write-Host ("  Baseline: {0} tokens across {1} files" -f $bench.baseline_tokens, $bench.baseline_files)
Write-Host ("  Slice:    {0} tokens across {1} files ({2:N1}% reduction)" -f $bench.slice_tokens, $bench.slice_files, $bench.reduction_pct)
foreach ($m in $bench.mcp) {
    Write-Host ("  MCP {0,-6}: {1,2} tools, {2,5} tokens" -f $m.profile, $m.tools, $m.tokens)
}

# 3. Inject 3 canonical AI-agent mistakes into a scratch copy
$TmpRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("clrinf_bench_" + [System.Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $TmpRoot -Force | Out-Null
try {
    $Case1Dir = Join-Path $TmpRoot "case1_forbid"
    Copy-Item -Path $BenchDir -Destination $Case1Dir -Recurse -Force
    $invoiceTs = Join-Path $Case1Dir "services\billing\src\domain\invoice.ts"
    Add-Content -Path $invoiceTs -Value "`nimport { Bad } from '../Infrastructure/db';`n"
    Push-Location $Case1Dir
    try {
        $out1 = & $Bin verify --json
        if ($LASTEXITCODE -eq 0) { throw "Case 1 should have failed verify" }
        $r1 = $out1 | ConvertFrom-Json
        if (-not ($r1.issues | Where-Object { $_.code -eq "RULE_FORBIDDEN_DEPENDENCY" })) {
            throw "Case 1 expected RULE_FORBIDDEN_DEPENDENCY, got: $out1"
        }
        Write-Host "[PASS] Case 1 caught forbidden domain import (RULE_FORBIDDEN_DEPENDENCY)" -ForegroundColor Green

        # Verify Claude hook exit code 2
        $payloadFile = Join-Path $TmpRoot "hook_payload.json"
        $hookErrFile = Join-Path $TmpRoot "hook_stderr.txt"
        [System.IO.File]::WriteAllText($payloadFile, '{"tool_input":{"file_path":"services/billing/src/domain/invoice.ts"}}')
        $proc = Start-Process -FilePath $Bin `
            -ArgumentList @("hook", "run", "--path", $Case1Dir, "--format", "claude") `
            -WorkingDirectory $Case1Dir `
            -RedirectStandardInput $payloadFile `
            -RedirectStandardError $hookErrFile `
            -NoNewWindow -Wait -PassThru
        $hookErr = if (Test-Path $hookErrFile) { Get-Content $hookErrFile -Raw } else { "" }
        if ($proc.ExitCode -ne 2 -or -not $hookErr.Contains("RULE_FORBIDDEN_DEPENDENCY")) {
            throw "Claude hook expected exit code 2 + RULE_FORBIDDEN_DEPENDENCY, got code=$($proc.ExitCode) stderr=$hookErr"
        }
        Write-Host "[PASS] Claude Code post-edit hook returned exit code 2 with actionable diagnostic" -ForegroundColor Green
    } finally {
        Pop-Location
    }

    $Case2Dir = Join-Path $TmpRoot "case2_outbox"
    Copy-Item -Path $BenchDir -Destination $Case2Dir -Recurse -Force
    $placeOrderRs = Join-Path $Case2Dir "services\orders\src\application\place_order.rs"
    Add-Content -Path $placeOrderRs -Value "`npub fn bypass(outbox: &mut Vec<String>) { outbox.enqueue(`"raw`".into()); }`n"
    Push-Location $Case2Dir
    try {
        $out2 = & $Bin verify --json
        if ($LASTEXITCODE -eq 0) { throw "Case 2 should have failed verify" }
        $r2 = $out2 | ConvertFrom-Json
        if (-not ($r2.issues | Where-Object { $_.code -eq "DRIFT_RAW_OUTBOX_WRITE" })) {
            throw "Case 2 expected DRIFT_RAW_OUTBOX_WRITE, got: $out2"
        }
        Write-Host "[PASS] Case 2 caught direct outbox bypass (DRIFT_RAW_OUTBOX_WRITE)" -ForegroundColor Green
    } finally {
        Pop-Location
    }

    $Case3Dir = Join-Path $TmpRoot "case3_upcaster"
    Copy-Item -Path $BenchDir -Destination $Case3Dir -Recurse -Force
    $v2Schema = @'
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.clrinf.com/orders/OrderPlaced.v2.schema.json",
  "title": "OrderPlacedV2",
  "type": "object",
  "x-domain": "orders",
  "x-event-type": "orders.placed.v2",
  "x-published-by": ["orders"],
  "x-subscribed-by": ["billing"],
  "required": ["order_id", "customer_id", "total_amount", "currency", "tax_amount"],
  "properties": {
    "order_id": { "type": "string" },
    "customer_id": { "type": "string" },
    "total_amount": { "type": "number" },
    "currency": { "type": "string" },
    "tax_amount": { "type": "number" }
  }
}
'@
    [System.IO.File]::WriteAllText(
        (Join-Path $Case3Dir "contracts\schemas\orders\OrderPlaced.v2.schema.json"),
        $v2Schema,
        [System.Text.UTF8Encoding]::new($false)
    )
    Push-Location $Case3Dir
    try {
        $out3 = & $Bin verify --json
        if ($LASTEXITCODE -eq 0) { throw "Case 3 should have failed verify" }
        $r3 = $out3 | ConvertFrom-Json
        if (-not ($r3.issues | Where-Object { $_.code -eq "TOPOLOGY_UPCASTER_MISSING" })) {
            throw "Case 3 expected TOPOLOGY_UPCASTER_MISSING, got: $out3"
        }
        Write-Host "[PASS] Case 3 caught breaking v2 evolution without upcaster (TOPOLOGY_UPCASTER_MISSING)" -ForegroundColor Green
    } finally {
        Pop-Location
    }

    # Case 4: cross-module internal import (boundary violation)
    $Case4Dir = Join-Path $TmpRoot "case4_boundary"
    Copy-Item -Path $BenchDir -Destination $Case4Dir -Recurse -Force
    $onOrderPlacedTs = Join-Path $Case4Dir "services\billing\src\application\on_order_placed.ts"
    Add-Content -Path $onOrderPlacedTs -Value "`nimport { Order } from '../../../orders/src/domain/order';`n"
    Push-Location $Case4Dir
    try {
        $out4 = & $Bin verify --json
        if ($LASTEXITCODE -eq 0) { throw "Case 4 should have failed verify" }
        $r4 = $out4 | ConvertFrom-Json
        if (-not ($r4.issues | Where-Object { $_.code -eq "RULE_MODULE_BOUNDARY_VIOLATION" })) {
            throw "Case 4 expected RULE_MODULE_BOUNDARY_VIOLATION, got: $out4"
        }
        Write-Host "[PASS] Case 4 caught cross-module internal import (RULE_MODULE_BOUNDARY_VIOLATION)" -ForegroundColor Green
    } finally {
        Pop-Location
    }

    # Case 5: editing a generated file caught by Claude post-edit hook
    $Case5Dir = Join-Path $TmpRoot "case5_generated"
    Copy-Item -Path $BenchDir -Destination $Case5Dir -Recurse -Force
    $genDir = Join-Path $Case5Dir "services\billing\src\generated"
    New-Item -ItemType Directory -Path $genDir -Force | Out-Null
    [System.IO.File]::WriteAllText(
        (Join-Path $genDir "order_placed.ts"),
        "// Code generated by clrinf-codegen. DO NOT EDIT.`nexport const editedByAgent = true;`n",
        [System.Text.UTF8Encoding]::new($false)
    )
    $genPayload = Join-Path $TmpRoot "hook_gen_payload.json"
    $genErrFile = Join-Path $TmpRoot "hook_gen_stderr.txt"
    [System.IO.File]::WriteAllText($genPayload, '{"tool_input":{"file_path":"services/billing/src/generated/order_placed.ts"}}')
    $genProc = Start-Process -FilePath $Bin `
        -ArgumentList @("hook", "run", "--path", $Case5Dir, "--format", "claude") `
        -WorkingDirectory $Case5Dir `
        -RedirectStandardInput $genPayload `
        -RedirectStandardError $genErrFile `
        -NoNewWindow -Wait -PassThru
    $genErr = if (Test-Path $genErrFile) { Get-Content $genErrFile -Raw } else { "" }
    if ($genProc.ExitCode -ne 2 -or -not $genErr.Contains("RULE_GENERATED_FILE_MODIFIED")) {
        throw "Case 5 expected exit code 2 + RULE_GENERATED_FILE_MODIFIED, got code=$($genProc.ExitCode) stderr=$genErr"
    }
    Write-Host "[PASS] Case 5 caught hand-edited generated file via Claude hook (RULE_GENERATED_FILE_MODIFIED)" -ForegroundColor Green

    Write-Host "`n=== Suite 2: Compile-Time AOT CRUD vs Disk-Generated CRUD Context Cost ===" -ForegroundColor Cyan

    $aotDomainDir = Join-Path $RepoRoot "core\csharp\examples\AotCrud\Domain"
    $aotFiles = Get-ChildItem $aotDomainDir -Filter "*.cs" -File
    $aotChars = ($aotFiles | ForEach-Object { (Get-Content $_.FullName -Raw).Length } | Measure-Object -Sum).Sum
    $aotTokens = [int][Math]::Ceiling($aotChars / 4.0)

    $DiskCrudDir = Join-Path $TmpRoot "disk_crud"
    New-Item -ItemType Directory -Path $DiskCrudDir -Force | Out-Null
    [System.IO.File]::WriteAllText(
        (Join-Path $DiskCrudDir "BenchApp.csproj"),
        '<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework></PropertyGroup></Project>',
        [System.Text.UTF8Encoding]::new($false)
    )
    & $Bin init --lang csharp --pattern flat --path $DiskCrudDir | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "clrinf init failed for Suite 2" }
    & $Bin add module Catalog --path $DiskCrudDir | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "clrinf add module Catalog failed for Suite 2" }
    & $Bin add entity Category -m Catalog --pattern vertical-slice --prop name:string --prop slug:string --templates-dir $TemplatesDir --path $DiskCrudDir | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "clrinf add entity Category failed for Suite 2" }
    & $Bin add entity Brand -m Catalog --pattern vertical-slice --prop name:string --templates-dir $TemplatesDir --path $DiskCrudDir | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "clrinf add entity Brand failed for Suite 2" }

    $diskFiles = Get-ChildItem $DiskCrudDir -Recurse -Filter "*.cs" -File | Where-Object { $_.FullName -notmatch '\\(bin|obj)\\' }
    $diskChars = ($diskFiles | ForEach-Object { (Get-Content $_.FullName -Raw).Length } | Measure-Object -Sum).Sum
    $diskTokens = [int][Math]::Ceiling($diskChars / 4.0)
    $crudReduction = if ($diskTokens -gt 0) { (1.0 - ($aotTokens / $diskTokens)) * 100.0 } else { 0.0 }

    Write-Host ("  Disk CRUD (vertical-slice, 2 entities): {0} tokens across {1} .cs files" -f $diskTokens, $diskFiles.Count)
    Write-Host ("  AOT CRUD on-disk (Domain/*, 2 entities): {0} tokens across {1} .cs files ({2:N1}% reduction)" -f $aotTokens, $aotFiles.Count, $crudReduction)

    Write-Host "`n=== Suite 3: Rust Compile-Time CRUD (#[derive(ClrinfCrud)]) vs Disk-Generated CRUD ===" -ForegroundColor Cyan

    $rustAotFiles = @(
        (Get-Item (Join-Path $RepoRoot "core\rust\examples\aot-crud\src\domain\brand.rs")),
        (Get-Item (Join-Path $RepoRoot "core\rust\examples\aot-crud\src\domain\category.rs"))
    )
    $rustAotChars = ($rustAotFiles | ForEach-Object { (Get-Content $_.FullName -Raw).Length } | Measure-Object -Sum).Sum
    $rustAotTokens = [int][Math]::Ceiling($rustAotChars / 4.0)

    $clrinfrsRelease = Join-Path $RepoRoot "core\rust\target\release\clrinfrs.exe"
    $clrinfrsDebug = Join-Path $RepoRoot "core\rust\target\debug\clrinfrs.exe"
    $clrinfrsExisting = @(@($clrinfrsRelease, $clrinfrsDebug) | Where-Object { Test-Path $_ } | Sort-Object { (Get-Item $_).LastWriteTimeUtc } -Descending)
    if ($clrinfrsExisting.Count -gt 0) {
        $RsBin = $clrinfrsExisting[0]
    } else {
        cargo build --offline --manifest-path (Join-Path $RepoRoot "core\rust\Cargo.toml") -p clrinf-cli | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "Failed to build clrinfrs for Suite 3" }
        $RsBin = $clrinfrsDebug
    }

    $RustDiskParent = Join-Path $TmpRoot "rust_disk_crud"
    New-Item -ItemType Directory -Path $RustDiskParent -Force | Out-Null
    & $RsBin new RustBenchApp --arch clean-cqrs --path $RustDiskParent | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "clrinfrs new failed for Suite 3" }
    $RustDiskApp = Join-Path $RustDiskParent "RustBenchApp"
    & $RsBin add module Catalog --project $RustDiskApp | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "clrinfrs add module Catalog failed for Suite 3" }
    & $RsBin add entity Category -m Catalog --prop name:string --prop slug:string --project $RustDiskApp | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "clrinfrs add entity Category failed for Suite 3" }
    & $RsBin add entity Brand -m Catalog --prop name:string --project $RustDiskApp | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "clrinfrs add entity Brand failed for Suite 3" }

    $rustDiskFiles = Get-ChildItem (Join-Path $RustDiskApp "modules\catalog") -Recurse -Filter "*.rs" -File
    $rustDiskChars = ($rustDiskFiles | ForEach-Object { (Get-Content $_.FullName -Raw).Length } | Measure-Object -Sum).Sum
    $rustDiskTokens = [int][Math]::Ceiling($rustDiskChars / 4.0)
    $rustReduction = if ($rustDiskTokens -gt 0) { (1.0 - ($rustAotTokens / $rustDiskTokens)) * 100.0 } else { 0.0 }

    $mapRaw = & $RsBin crud-map (Join-Path $RepoRoot "core\rust\examples\aot-crud") --json
    if ($LASTEXITCODE -ne 0) { throw "clrinfrs crud-map failed for Suite 3" }
    $mapJson = $mapRaw | ConvertFrom-Json
    if ($mapJson.entities.Count -lt 2) { throw "Expected >=2 #[derive(ClrinfCrud)] entities in aot-crud" }
    Write-Host ("  Rust Disk CRUD (clrinfrs add entity, 2 entities): {0} tokens across {1} .rs files" -f $rustDiskTokens, $rustDiskFiles.Count)
    Write-Host ("  Rust Compile-Time CRUD (domain/*, 2 entities):   {0} tokens across {1} .rs files ({2:N1}% reduction)" -f $rustAotTokens, $rustAotFiles.Count, $rustReduction)
} finally {
    Remove-Item -Path $TmpRoot -Recurse -Force -ErrorAction SilentlyContinue
}
