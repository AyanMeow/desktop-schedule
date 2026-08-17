# 发版辅助：提取本版公告 + 计算 SHA-256 + 发布 GitHub 与 Gitee 双源 Release
param([Parameter(Mandatory)][string]$Ver)
$ErrorActionPreference = 'Stop'

# Clash 代理（本机 GitHub 访问必需；Gitee 走直连）
$env:HTTP_PROXY = 'http://127.0.0.1:7899'
$env:HTTPS_PROXY = 'http://127.0.0.1:7899'
$gh = 'C:\Program Files\GitHub CLI\gh.exe'

# 1) 提取更新公告中 "## vX.Y.Z" 到下一个 "---" 的段落
$lines = Get-Content '更新公告.md' -Encoding UTF8
$hit = $lines | Select-String -SimpleMatch "## v$Ver" | Select-Object -First 1
if (-not $hit) { throw "更新公告.md 中未找到 ## v$Ver 段落" }
$start = $hit.LineNumber  # 1-based 行号，即段落首行
$end = $lines.Count
for ($i = $start; $i -lt $lines.Count; $i++) {
  if ($lines[$i] -match '^---\s*$') { $end = $i; break }
}
$section = ($lines[($start)..($end - 1)] -join "`n").Trim()

# 2) 计算 exe SHA-256
$exe = Resolve-Path 'desktop-schedule\src-tauri\target\release\desktop-schedule.exe'
$hash = (Get-FileHash $exe -Algorithm SHA256).Hash.ToLower()
Write-Output "SHA256: $hash"

# 3) notes = 本版公告 + 哈希行（更新器靠 "SHA256: <hash>" 行校验下载）
$notes = $section + "`n`n`nSHA256: $hash`n"
$tmp = Join-Path $env:TEMP 'release-notes.md'
[IO.File]::WriteAllText($tmp, $notes, [Text.UTF8Encoding]::new($false))

# 4) GitHub Release（tag + 附 exe）
& $gh release create "v$Ver" $exe --title "v$Ver" --notes-file $tmp
if ($LASTEXITCODE -ne 0) { throw "gh release create 失败" }
Write-Output "GitHub Release v$Ver 创建成功"

# 5) Gitee Release（国内更新源，直连不走代理；令牌读 .gitee-token）
$tokenPath = Join-Path $PSScriptRoot '.gitee-token'
if (Test-Path $tokenPath) {
  $gtoken = (Get-Content $tokenPath -Raw).Trim()
  $api = 'https://gitee.com/api/v5/repos/ayanmeow/desktop-schedule'
  # 创建 Release（tag 不存在时按 main 创建）
  $createJson = & curl.exe -s --noproxy '*' -X POST "$api/releases" `
    -d "access_token=$gtoken" -d "tag_name=v$Ver" -d "name=v$Ver" `
    -d "target_commitish=main" `
    --data-urlencode "body@$tmp" 2>$null
  $rel = $null
  try { $rel = $createJson | ConvertFrom-Json } catch {}
  if (-not $rel -or -not $rel.id) {
    Write-Output "Gitee Release 创建失败：$($createJson | Out-String)"
  } else {
    # 上传 exe 附件
    $attachJson = & curl.exe -s --noproxy '*' -X POST "$api/releases/$($rel.id)/attach_files" `
      -F "access_token=$gtoken" -F "files=@$exe" 2>$null
    $att = $null
    try { $att = @($attachJson | ConvertFrom-Json) } catch {}
    if ($att -and $att[0].browser_download_url) {
      Write-Output "Gitee Release v$Ver 创建成功（附件已上传）"
    } else {
      Write-Output "Gitee 附件上传异常：$($attachJson | Out-String)"
    }
  }
} else {
  Write-Output "未找到 .gitee-token，跳过 Gitee 发布"
}
