# 发版辅助：从更新公告提取本版段落 + 计算 SHA-256 + 创建 GitHub Release
param([Parameter(Mandatory)][string]$Ver)
$ErrorActionPreference = 'Stop'

# Clash 代理（本机 GitHub 访问必需）
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
$exe = 'desktop-schedule\src-tauri\target\release\desktop-schedule.exe'
if (-not (Test-Path $exe)) { throw "找不到 $exe（先完成构建）" }
$hash = (Get-FileHash $exe -Algorithm SHA256).Hash.ToLower()
Write-Output "SHA256: $hash"

# 3) notes = 本版公告 + 哈希行（更新器靠 "SHA256: <hash>" 行校验下载）
$notes = $section + "`n`n`nSHA256: $hash`n"
$tmp = Join-Path $env:TEMP 'release-notes.md'
[IO.File]::WriteAllText($tmp, $notes, [Text.UTF8Encoding]::new($false))

# 4) 创建 Release（tag v版本号 + 附 exe）
& $gh release create "v$Ver" $exe --title "v$Ver" --notes-file $tmp
if ($LASTEXITCODE -ne 0) { throw "gh release create 失败" }
Write-Output "Release v$Ver 创建成功"
