<#
.SYNOPSIS
  Instrumented SCIP-indexing probe for the corpus-scale campaign.

.DESCRIPTION
  Runs `rust-analyzer scip` against a target repo and records a machine-readable PROFILE of how it went —
  per-stage wall time, peak working set, exit status, and output size — preserving partial output on failure.

  This exists because breakage IS the finding. `cross_corpus.rs` shells out to the same command with
  `assert!(status.success())`, so a failure panics and captures nothing: we learn "it died". This probe is
  built to learn "it died at stage <s>, <t> minutes in, at <m> GB, after emitting <n> bytes" — the same
  failure, with the information that makes it worth the compute.

  The cost/failure profile is not overhead: it is input to antigen's observability/config tier (which
  instruments a user can afford to enable at what scale), and promotes config entries from "reasoned" to
  "PROVEN". See jbd charter `corpus-scale-probe`.

  CALIBRATE ON A MID-SIZE TARGET FIRST. Do not spend a rustc-sized run proving the harness records nothing.

.PARAMETER Repo
  Path to the target repository (must contain a Cargo workspace).

.PARAMETER Out
  Directory for the emitted index + profile. Defaults to <Repo>/target.

.PARAMETER TimeoutMinutes
  Abort the indexing stage after this many minutes and record a timeout outcome (default 120).
  A timeout is a RESULT, not an error — it is the cost datum we are collecting.

.EXAMPLE
  pwsh scripts/scale-probe.ps1 -Repo R:/corpora/cargo -TimeoutMinutes 60
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Repo,
    [string]$Out,
    [int]$TimeoutMinutes = 120
)

$ErrorActionPreference = 'Stop'
if (-not (Test-Path $Repo)) { throw "repo not found: $Repo" }
$Repo = (Resolve-Path $Repo).Path
if (-not $Out) { $Out = Join-Path $Repo 'target' }
if (-not (Test-Path $Out)) { New-Item -ItemType Directory -Force $Out | Out-Null }

$profile = [ordered]@{
    target          = $Repo
    started_utc     = (Get-Date).ToUniversalTime().ToString('o')
    tool            = $null
    target_commit   = $null
    workspace_files = $null
    stages          = @()
    outcome         = 'incomplete'
}

function Add-Stage([string]$name, [scriptblock]$work) {
    Write-Host "[stage] $name ..." -ForegroundColor Cyan
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $status = 'ok'; $detail = $null
    try { $detail = & $work } catch { $status = 'failed'; $detail = $_.Exception.Message }
    $sw.Stop()
    $script:profile.stages += [ordered]@{
        name        = $name
        seconds     = [math]::Round($sw.Elapsed.TotalSeconds, 2)
        status      = $status
        detail      = $detail
    }
    Write-Host ("[stage] {0}: {1} ({2}s)" -f $name, $status, [math]::Round($sw.Elapsed.TotalSeconds, 1))
    if ($status -eq 'failed') { throw "stage '$name' failed: $detail" }
    return $detail
}

try {
    # -- provenance: what exactly did we measure? A profile without this is not comparable across runs.
    $profile.tool = (& rust-analyzer --version) 2>&1 | Out-String
    $profile.tool = $profile.tool.Trim()
    Add-Stage 'provenance' {
        try { $script:profile.target_commit = (& git -C $Repo rev-parse HEAD 2>$null).Trim() } catch {}
        $n = (Get-ChildItem -Path $Repo -Filter *.rs -Recurse -File -ErrorAction SilentlyContinue |
              Measure-Object).Count
        $script:profile.workspace_files = $n
        "rs files: $n"
    }

    # -- the load-bearing stage. Timeout and non-zero exit are RESULTS, recorded, never thrown away.
    $scip = Join-Path $Out 'index.scip'
    if (Test-Path $scip) { Remove-Item $scip -Force }
    $stdout = Join-Path $Out 'scale-probe.stdout.log'
    $stderr = Join-Path $Out 'scale-probe.stderr.log'

    Add-Stage 'rust-analyzer-scip' {
        $p = Start-Process -FilePath 'rust-analyzer' -ArgumentList @('scip', '.') `
             -WorkingDirectory $Repo -PassThru -NoNewWindow `
             -RedirectStandardOutput $stdout -RedirectStandardError $stderr
        # NOTE (calibration finding): `.cargo/bin/rust-analyzer` is a 10-byte rustup PROXY. Sampling the
        # spawned handle measures the SHIM (~10 MB) and silently reports a plausible-but-wrong peak. Sample
        # the whole rust-analyzer process GROUP instead, and take the max of the SUM of working sets.
        $peak = 0L
        $deadline = (Get-Date).AddMinutes($TimeoutMinutes)
        while (-not $p.HasExited) {
            try {
                $sum = (Get-Process -Name 'rust-analyzer' -ErrorAction SilentlyContinue |
                        Measure-Object -Property WorkingSet64 -Sum).Sum
                if ($sum -and $sum -gt $peak) { $peak = [int64]$sum }
            } catch {}
            if ((Get-Date) -gt $deadline) {
                try { $p.Kill($true) } catch {}
                $script:profile.peak_rss_gb = [math]::Round($peak / 1GB, 3)
                $script:profile.outcome = 'timeout'
                return "TIMEOUT after $TimeoutMinutes min; peak RSS $([math]::Round($peak/1GB,2)) GB"
            }
            Start-Sleep -Milliseconds 500
        }
        $script:profile.peak_rss_gb = [math]::Round($peak / 1GB, 3)
        $script:profile.exit_code = $p.ExitCode
        if ($p.ExitCode -ne 0) { $script:profile.outcome = 'indexer-nonzero-exit' }
        "exit $($p.ExitCode); peak RSS $([math]::Round($peak/1GB,2)) GB"
    }

    # -- partial output is still data. Never discard what was emitted before a failure.
    # NOTE (calibration finding): `rust-analyzer scip .` writes `index.scip` into its WORKING DIRECTORY
    # (the repo root), NOT into any -Out path. Looking only at $Out reported "no index emitted" for a run
    # that had emitted 14.5 MB. Search both, then relocate so the profile and its artifact travel together.
    Add-Stage 'output-census' {
        $emitted = @($scip, (Join-Path $Repo 'index.scip')) | Where-Object { Test-Path $_ } | Select-Object -First 1
        if ($emitted) {
            $sz = (Get-Item $emitted).Length
            if ($emitted -ne $scip) { Move-Item -Force $emitted $scip }
            $script:profile.index_bytes = $sz
            $script:profile.index_path = $scip
            "index.scip: $([math]::Round($sz/1MB,2)) MB (from $emitted)"
        } else {
            $script:profile.index_bytes = 0
            'no index emitted (partial logs retained)'
        }
    }

    if ($profile.outcome -eq 'incomplete') { $profile.outcome = 'ok' }
}
catch {
    if ($profile.outcome -eq 'incomplete') { $profile.outcome = 'failed' }
    $profile.error = $_.Exception.Message
    Write-Host "[probe] recorded failure: $($_.Exception.Message)" -ForegroundColor Yellow
}
finally {
    $profile.finished_utc = (Get-Date).ToUniversalTime().ToString('o')
    $profilePath = Join-Path $Out 'scale-probe.profile.json'
    $profile | ConvertTo-Json -Depth 6 | Set-Content -Path $profilePath -Encoding utf8
    Write-Host "[probe] outcome=$($profile.outcome)  profile -> $profilePath" -ForegroundColor Green
}
