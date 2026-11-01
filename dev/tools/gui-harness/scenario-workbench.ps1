# Runs dev/apps/workbench/gui_scenario.json (path in $env:SCENARIO_JSON).
# Driven by gui.ps1: gui.ps1 -Exe ...\workbench.exe -Title "Kivact Workbench" -Out <dir> -Scenario workbench
. (Join-Path $PSScriptRoot "ext-helpers.ps1")
$spec = Get-Content -Raw -Encoding UTF8 $env:SCENARIO_JSON | ConvertFrom-Json
foreach ($step in $spec.steps) {
    Log "=== step $($step.name)"
    foreach ($a in $step.actions) {
        $id = $a.automationId; $name = $a.name; $ctl = $a.control
        $desc = "$($a.kind) id=$id name=$name"
        switch ($a.kind) {
            "require" { Check $desc (El-Wait $id $name $ctl $a.value) }
            "wait" { Check $desc (El-Wait $id $name $ctl $a.value 100) }
            "absent" { Check $desc ((El-Where $id $name $ctl).Count -eq 0) }
            "wait_absent" { Check $desc (El-WaitAbsent $id $name $ctl 100) }
            "click" { El-Click (El-One $id $name $ctl) ; Log "  clicked" }
            "invoke" { El-Invoke (El-One $id $name $ctl); Log "  invoked" }
            "wait_focus" {
                $ok = $false
                for ($k = 0; $k -lt 30; $k++) { if ((Focused-Id) -match $id) { $ok = $true; break }; Start-Sleep -Milliseconds 100 }
                Check "$desc focused='$(Focused-Id)'" $ok
            }
            "type" { [W]::Text($a.text); Start-Sleep -Milliseconds 200 }
            "key" { [W]::Key([uint16]$a.vk); Start-Sleep -Milliseconds 200 }
            "assert_value" {
                $ok = $false; $v = $null
                for ($k = 0; $k -lt 30; $k++) { $v = El-Value (El-One $id $name $ctl); if ($v -match $a.value) { $ok = $true; break }; Start-Sleep -Milliseconds 100 }
                Check "$desc value='$v' ~ $($a.value)" $ok
            }
            "wheel" {
                $c = El-Center (El-One $id $name $ctl)
                $n = if ($a.repeat) { [int]$a.repeat } else { 1 }
                for ($k = 0; $k -lt $n; $k++) { [W]::Wheel($c[0], $c[1], [int]$a.delta); Start-Sleep -Milliseconds 40 }
                Start-Sleep -Milliseconds 400
                Log "  wheel x$n delta=$($a.delta)"
            }
            "drag" { El-DragX (El-One $id $name $ctl) ([double]$a.from) ([double]$a.to); Log "  dragged $($a.from) -> $($a.to)" }
            "require_count_max" {
                $n = (El-Where -Control $ctl).Count
                Check "$desc count=$n <= $($a.max)" ($n -le [int]$a.max -and $n -gt 0)
            }
            default { Log "  unknown action $($a.kind)"; $script:failed++ }
        }
        Log "  focus after $($a.kind): id=$(Focused-Id) name=$(Focused-Name)"
    }
    if ($step.capture) { Shot ("wb-" + $step.name) | Out-Null }
}
Dump "final"
