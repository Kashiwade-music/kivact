# Mini Postman real-window scenario against a local Python HTTP server.
param([string]$Exe, [string]$Out, [int]$Round = 1, [int]$Port = 8765)
. (Join-Path $PSScriptRoot "apps-lib.ps1")
New-Item -ItemType Directory -Force $Out | Out-Null
$script:out = $Out
$script:log = Join-Path $Out "log.txt"
$title = "Kivact · Mini Postman"
$history = Join-Path $Out "history.json"
Remove-Item -Force $history -ErrorAction SilentlyContinue
$env:KIVACT_POSTMAN_HISTORY = $history
$serverLog = Join-Path $Out "http_server.log"
Remove-Item -Force $serverLog -ErrorAction SilentlyContinue
$server = Start-Process -FilePath python -ArgumentList @((Join-Path $PSScriptRoot "http_server.py"), $Port, $serverLog) -PassThru -WindowStyle Hidden
Start-Sleep -Milliseconds 800
$base = "http://127.0.0.1:$Port"
function SavedHistory() {
    if (-not (Test-Path $history)) { return @() }
    $j = Read-AtomicJsonSnapshot $history
    if ($null -eq $j) { return @() }
    return $j
}
function CurlCount() { return @(Get-Process curl -ErrorAction SilentlyContinue).Count }
function StatusLike($pattern) { return (WaitNameLike "postman-status" $pattern 100) }

Enter-GuiLock "scenario-postman round $Round"
try {
    Log "=== Mini Postman round $Round $(Get-Date -Format o)"
    Log "exe sha256=$((Get-FileHash $Exe -Algorithm SHA256).Hash)"
    Start-App $Exe $title
    Check "initial status" (WaitName "postman-status" "送信待ち")
    Check "cancel is disabled while idle" (-not (Find "cancel").Current.IsEnabled)

    # 1. Invalid URL is reported (no request)
    TypeInto "url" "invalid-url"
    ClickId "send"
    Check "invalid URL message" (StatusLike "URLは http://*")

    # 2. Real GET (mouse + SendInput text)
    TypeInto "url" "$base/hello?from=gui"
    Check "URL typed (UIA value)" ((ValueOf "url") -eq "$base/hello?from=gui")
    ClickId "send"
    Check "GET succeeds" (StatusLike "HTTP 200 · * ms")
    Check "response shows headers and body" (WaitNameLike "response" '*X-Server: kivact-gui-test*"hello": "kivact"*')
    Check "history row added" (WaitNameLike "history-1" "GET $base/hello?from=gui — HTTP 200*")
    Shot "pm-get" | Out-Null

    # 3. Copy the response to the OS clipboard
    ClickId "copy-response"
    Check "copy reported" (WaitName "postman-status" "応答をコピーしました")
    $clip = Get-Clipboard -Raw
    Check "OS clipboard has the response" ($clip -like '*"hello": "kivact"*')

    # 4. POST with the method select (keyboard, without opening the popup;
    #    check-postman.ps1 also uses the mouse), a header and a Japanese body
    ClickId "url"
    [AW]::KeyCombo(0x10, 0x09); Start-Sleep -Milliseconds 200
    Check "Shift+Tab from URL focuses the method select" ((Focused) -eq "method")
    [AW]::Key(0x28)  # ArrowDown chooses POST
    Start-Sleep -Milliseconds 300
    Check "method is POST" ((ValueOf "method") -eq "POST")
    TypeInto "url" "$base/echo"
    TypeInto "headers" "X-Test: gui-run"
    TypeInto "request-body" "こんにちは body"
    ClickId "send"
    Check "POST returns 201" (StatusLike "HTTP 201 · * ms")
    Check "echoed body and header" (WaitNameLike "response" '*"echo": "こんにちは body"*"x_test": "gui-run"*')
    $srv = Get-Content -Encoding UTF8 $serverLog
    Check "server received the POST body" (($srv | Where-Object { $_ -like "*POST /echo header=gui-run body='こんにちは body'*" }).Count -ge 1)

    # 5. Slow request: pending state, cancel stops it
    ClickId "url"; [AW]::KeyCombo(0x10, 0x09); Start-Sleep -Milliseconds 200; [AW]::Key(0x26); Start-Sleep -Milliseconds 300
    Check "method back to GET" ((ValueOf "method") -eq "GET")
    TypeInto "url" "$base/slow"
    ClickId "send"
    Check "pending status" (WaitName "postman-status" "送信中")
    Check "cancel enabled while pending" ((Find "cancel").Current.IsEnabled)
    Start-Sleep -Milliseconds 500
    $curls = CurlCount
    Log "  curl processes while pending: $curls"
    Check "a curl process runs while pending" ($curls -ge 1)
    ClickId "cancel"
    Check "cancelled status" (WaitName "postman-status" "キャンセルしました")
    for ($k = 0; $k -lt 20 -and (CurlCount) -gt 0; $k++) { Start-Sleep -Milliseconds 100 }
    Check "cancel stops the curl process" ((CurlCount) -eq 0)
    Check "cancel disabled again" (-not (Find "cancel").Current.IsEnabled)
    Start-Sleep -Milliseconds 6500
    $srv = Get-Content -Encoding UTF8 $serverLog
    Log "  server after the slow request: $(($srv | Select-Object -Last 2) -join ' | ')"
    Check "cancelled request is not in the history" ((SavedHistory).Count -eq 2)

    # 6. Latest wins: slow, then fast before the slow one finishes
    ClickId "send"
    Check "slow pending again" (WaitName "postman-status" "送信中")
    TypeInto "url" "$base/hello?latest=1"
    ClickId "send"
    for ($k = 0; $k -lt 20 -and (CurlCount) -gt 0; $k++) { Start-Sleep -Milliseconds 100 }
    Check "the superseded curl process is stopped" ((CurlCount) -eq 0)
    Check "fast response wins" (StatusLike "HTTP 200 · * ms")
    Start-Sleep -Milliseconds 6500
    Check "status still shows the fast response" (StatusLike "HTTP 200 · * ms")
    $h = SavedHistory
    Log "  history: $(($h | ForEach-Object { $_.request.url }) -join ', ')"
    Check "only the latest request entered the history" ($h.Count -eq 3 -and $h[0].request.url -eq "$base/hello?latest=1")

    # 7. Retry re-runs the last request
    ClickId "retry"
    Check "retry succeeds" (StatusLike "HTTP 200 · * ms")
    Check "retry added a history row" ((WaitNameLike "history-4" "GET $base/hello?latest=1*"))

    # 8. Focus the URL field through NodeRef
    ClickId "send"
    Start-Sleep -Milliseconds 300
    ClickId "focus-url"
    Check "focus message" (WaitName "postman-status" "URL欄へ移動しました")
    Check "URL field has keyboard focus" ((Focused) -eq "url")

    # 9. History row restores the POST form
    ClickId "history-2"
    Check "history restores method" ((ValueOf "method") -eq "POST")
    Check "history restores URL" ((ValueOf "url") -eq "$base/echo")
    Check "history restores headers and body" (((ValueOf "headers") -eq "X-Test: gui-run") -and ((ValueOf "request-body") -eq "こんにちは body"))
    Shot "pm-restored" | Out-Null

    # 10. Restart: the history is loaded again
    $code = Stop-App
    Check "graceful close exits with code 0" ($code -eq 0)
    $n = (SavedHistory).Count
    Start-App $Exe $title
    Check "history restored after restart" (WaitNameLike "history-$n" "GET $base/hello?latest=1*")
    Check "restored POST row" (WaitNameLike "history-2" "POST $base/echo — HTTP 201*")
    # Retry without a run in this session sends the newest history entry
    ClickId "retry"
    Check "retry from history works after restart" (StatusLike "HTTP 200 · * ms")
    $code = Stop-App
    Check "second close exits with code 0" ($code -eq 0)
} catch {
    Log "ERROR $_ $($_.ScriptStackTrace)"
    $script:failed++
    Kill-App
} finally {
    Exit-GuiLock
    Stop-Process -Id $server.Id -Force -ErrorAction SilentlyContinue
    Log "passed=$script:passed failed=$script:failed"
}
exit $script:failed
