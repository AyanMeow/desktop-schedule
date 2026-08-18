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
  $notesText = Get-Content $tmp -Raw

  # 创建 Release（tag 不存在时按 main 创建）；重复执行时按 tag 取现有 Release（幂等）
  $rel = $null
  try {
    $rel = Invoke-RestMethod -Method Post -Uri "$api/releases" -Body @{
      access_token = $gtoken; tag_name = "v$Ver"; name = "v$Ver"
      target_commitish = 'main'; body = $notesText
    } -TimeoutSec 60
    Write-Output "Gitee Release v$Ver 已创建"
  } catch {
    try {
      $rel = Invoke-RestMethod -Method Get -Uri "$api/releases/tags/v$Ver`?access_token=$gtoken" -TimeoutSec 30
      Write-Output "Gitee Release v$Ver 已存在（幂等复用）"
    } catch { $rel = $null }
  }
  if (-not $rel -or -not $rel.id) { throw "Gitee Release 创建与查询均失败" }

  # 附件已存在则跳过上传（完全幂等）
  $existing = Invoke-RestMethod -Method Get -Uri "$api/releases/tags/v$Ver`?access_token=$gtoken" -TimeoutSec 30
  $hasExe = @($existing.assets | Where-Object { $_.name -eq 'desktop-schedule.exe' }).Count -gt 0
  if (-not $hasExe) {
    # 上传 exe 附件（注意：Gitee 字段名是单数 file，文档写的 files 实际无效）
    $null = & curl.exe -s --noproxy '*' -X POST "$api/releases/$($rel.id)/attach_files" `
      -F "access_token=$gtoken" -F "file=@$exe" 2>$null
    # 验证附件确实存在，缺失视为发版失败（throw 中断，让问题当场暴露）
    $verify = Invoke-RestMethod -Method Get -Uri "$api/releases/tags/v$Ver`?access_token=$gtoken" -TimeoutSec 30
    $hasExe = @($verify.assets | Where-Object { $_.name -eq 'desktop-schedule.exe' }).Count -gt 0
    if (-not $hasExe) { throw "Gitee 附件上传失败：Release 中缺少 desktop-schedule.exe" }
  }
  Write-Output "Gitee Release v$Ver 就绪（附件已确认）"
} else {
  Write-Output "未找到 .gitee-token，跳过 Gitee 发布"
}
