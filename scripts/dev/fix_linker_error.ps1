# Fix linker error LNK1104 script
# Usage: .\scripts\dev\fix_linker_error.ps1

if (-not $PSScriptRoot) {
    $PSScriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
}

Write-Host "Fixing linker error LNK1104..." -ForegroundColor Cyan

# Step 1: Kill related processes
Write-Host "`n[1/4] Checking and killing related processes..." -ForegroundColor Yellow
$processes = Get-Process | Where-Object {
    $_.ProcessName -like "*cargo*" -or 
    $_.ProcessName -like "*rustc*" -or 
    $_.ProcessName -like "*module_test*" -or
    $_.ProcessName -like "*workflow*"
}

if ($processes) {
    Write-Host "Found processes, killing..." -ForegroundColor Yellow
    $processes | ForEach-Object {
        Write-Host "  - $($_.ProcessName) (PID: $($_.Id))" -ForegroundColor Gray
        try {
            Stop-Process -Id $_.Id -Force -ErrorAction Stop
        } catch {
            Write-Host "    Warning: Cannot kill process $($_.ProcessName): $_" -ForegroundColor Red
        }
    }
    Start-Sleep -Seconds 2
} else {
    Write-Host "No related processes found" -ForegroundColor Green
}

# Step 2: Check and unlock target files
Write-Host "`n[2/4] Checking target file lock status..." -ForegroundColor Yellow
$targetDir = Join-Path $PSScriptRoot "..\..\target"
if (Test-Path $targetDir) {
    $exeFiles = Get-ChildItem -Path $targetDir -Recurse -Filter "*.exe" -ErrorAction SilentlyContinue
    foreach ($file in $exeFiles) {
        try {
            $stream = [System.IO.File]::Open($file.FullName, 'Open', 'ReadWrite', 'None')
            $stream.Close()
        } catch {
            Write-Host "  Warning: File may be locked: $($file.FullName)" -ForegroundColor Yellow
            Write-Host "    Trying to delete..." -ForegroundColor Gray
            try {
                Remove-Item $file.FullName -Force -ErrorAction Stop
                Write-Host "    Deleted" -ForegroundColor Green
            } catch {
                Write-Host "    Cannot delete, may need manual handling" -ForegroundColor Red
            }
        }
    }
} else {
    Write-Host "target directory does not exist, skipping" -ForegroundColor Gray
}

# Step 3: Clean build cache
Write-Host "`n[3/4] Cleaning build cache..." -ForegroundColor Yellow
$cargoPath = Get-Command cargo -ErrorAction SilentlyContinue
if ($cargoPath) {
    try {
        Push-Location (Join-Path $PSScriptRoot "..\..")
        cargo clean
        Write-Host "Build cache cleaned" -ForegroundColor Green
        Pop-Location
    } catch {
        Write-Host "Warning: cargo clean failed: $_" -ForegroundColor Red
        Pop-Location
    }
} else {
    Write-Host "Warning: cargo command not found" -ForegroundColor Red
}

# Step 4: Verify fix
Write-Host "`n[4/4] Verifying fix..." -ForegroundColor Yellow
$targetDir = Join-Path $PSScriptRoot "..\..\target"
if (Test-Path $targetDir) {
    $canWrite = $false
    try {
        $testFile = Join-Path $targetDir "test_write.tmp"
        "test" | Out-File $testFile -ErrorAction Stop
        Remove-Item $testFile -ErrorAction Stop
        $canWrite = $true
    } catch {
        Write-Host "Warning: Cannot write to target directory, may have permission issues" -ForegroundColor Red
    }
    
    if ($canWrite) {
        Write-Host "target directory is writable" -ForegroundColor Green
    }
} else {
    Write-Host "target directory does not exist (normal, will be created on next build)" -ForegroundColor Gray
}

Write-Host "`nFix completed!" -ForegroundColor Green
Write-Host "`nSuggested next steps:" -ForegroundColor Cyan
Write-Host "  1. If problem persists, check if antivirus locked the files" -ForegroundColor White
Write-Host "  2. Add target directory to antivirus exclusion list" -ForegroundColor White
Write-Host "  3. Try single-threaded compilation: `$env:CARGO_BUILD_JOBS='1'; cargo test ..." -ForegroundColor White
Write-Host "  4. Re-run tests: cargo test --test module_test" -ForegroundColor White
