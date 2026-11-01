# Build reviewable image pairs without assigning a synthetic pixel-perfect score.
# Source capture byte dimensions may differ from its recorded CSS viewport.
param(
    [Parameter(Mandatory=$true)][string]$Source,
    [Parameter(Mandatory=$true)][string]$Rendered,
    [Parameter(Mandatory=$true)][string]$Out,
    [string]$CodeCommit = ''
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$Source = (Resolve-Path -LiteralPath $Source).Path
$Rendered = (Resolve-Path -LiteralPath $Rendered).Path
New-Item -ItemType Directory -Force -Path $Out | Out-Null
$Out = (Resolve-Path -LiteralPath $Out).Path
$sourceConditions = @()
$sourceConditionFile = Join-Path $Source 'conditions.json'
if (Test-Path -LiteralPath $sourceConditionFile) {
    $sourceConditions = @(Get-Content -Raw -Encoding UTF8 -LiteralPath $sourceConditionFile | ConvertFrom-Json)
}
$chapters = @{
    basic = @('initial','overview','tokens','buttons','forms','selection','display','table','feedback','overlays','navigation','messaging','schedule','cards')
    game = @('initial','foundations','buttons','inputs','selection','navigation','display','table','feedback','overlays','layout')
    creative = @('initial','buttons','numeric','selection','indicators','transport','knobs','meters','mixer','properties','docking','outliner','media','layers','timeline','animation','nodes','grading','picker','viewport','menus','dialogs','jobs')
}
$pairs = @()
$html = [Collections.Generic.List[string]]::new()
$html.Add('<!doctype html><meta charset="utf-8"><title>Official UI comparison</title><style>body{margin:24px;background:#151519;color:#eee;font:14px system-ui}h2{position:sticky;top:0;background:#151519;padding:12px}article{display:block;margin-bottom:32px}img{width:100%;height:auto;display:block}figcaption{padding:8px;color:#bbb}figure{margin:0}</style><h1>Source / Kivact</h1><p>45 source chapters, plus three initial screens. Compare the recorded viewport, theme and interaction state. Capture scaling and rasterization differences are recorded in manifest.json.</p>')
$labelFont = [Drawing.Font]::new('Segoe UI',14)
try {
    foreach ($scene in @('basic','game','creative')) {
        foreach ($chapter in $chapters[$scene]) {
            $name = "$scene-$chapter"
            $sourceFile = Join-Path $Source "$name.jpg"
            $renderFile = Join-Path $Rendered "$name.png"
            if (-not (Test-Path -LiteralPath $sourceFile) -or -not (Test-Path -LiteralPath $renderFile)) {
                throw "Missing comparison input: $name"
            }
            $left = [Drawing.Image]::FromFile($sourceFile)
            $right = [Drawing.Image]::FromFile($renderFile)
            $condition = $sourceConditions | Where-Object { $_.scene -eq $scene -and $_.chapter -eq $chapter } | Select-Object -First 1
            $renderCondition = $null
            $renderConditionFile = Join-Path $Rendered "$scene-conditions.json"
            if (Test-Path -LiteralPath $renderConditionFile) {
                $renderCondition = Get-Content -Raw -Encoding UTF8 -LiteralPath $renderConditionFile | ConvertFrom-Json
            }
            $pair = [Drawing.Bitmap]::new($left.Width + $right.Width + 16, [Math]::Max($left.Height,$right.Height) + 36)
            $graphics = [Drawing.Graphics]::FromImage($pair)
            try {
                $graphics.Clear([Drawing.Color]::FromArgb(21,21,25))
                $graphics.DrawString("SOURCE: $name | image $($left.Width)x$($left.Height) | CSS $($condition.width)x$($condition.height)",$labelFont,[Drawing.Brushes]::White,8,4)
                $graphics.DrawString("KIVACT: $name | image/client $($right.Width)x$($right.Height)",$labelFont,[Drawing.Brushes]::White,$left.Width+24,4)
                $graphics.DrawImageUnscaled($left,0,36)
                $graphics.DrawImageUnscaled($right,$left.Width+16,36)
                $pair.Save((Join-Path $Out "$name.png"))
                $pairs += @{name=$name;source=$sourceFile;sourceSha256=(Get-FileHash -LiteralPath $sourceFile).Hash;sourcePixels=@($left.Width,$left.Height);sourceCondition=$condition;rendered=$renderFile;renderedSha256=(Get-FileHash -LiteralPath $renderFile).Hash;renderedPixels=@($right.Width,$right.Height);renderedCondition=$renderCondition;resampled=$false}
            } finally { $graphics.Dispose();$pair.Dispose();$left.Dispose();$right.Dispose() }
            $html.Add("<h2>$name</h2><article><figure><img src='$name.png'><figcaption>Source on the left, Kivact on the right; original image bytes are drawn without resampling.</figcaption></figure></article>")
        }
    }
} finally { $labelFont.Dispose() }
@{chapters=45;initialScreens=3;renderedCodeCommit=$CodeCommit;comparisonPairs=$pairs;method='side by side; no resizing, no numerical visual acceptance score'} | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $Out 'manifest.json') -Encoding UTF8
$html -join "`n" | Set-Content -LiteralPath (Join-Path $Out 'index.html') -Encoding UTF8
Write-Output "Wrote $($pairs.Count) image pairs to $Out"
