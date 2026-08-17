# 桌面日程 · Desktop Schedule

Windows 桌面常驻的半透明日程管理小组件——无边框贴片"贴"在桌面上（非置顶、可被窗口遮挡），像桌面的一部分，随手查看与管理日程。

## 核心特性

- **日历视图**：周 / 双周 / 月切换，点击日期展开当日日程
- **完成激励**：勾选完成 → 删除线 + 随机鼓励语 + 艾露猫表情包气泡
- **成就系统**：19 枚成就（累计完成 / 连续坚持 / 单日爆发），未解锁显示 ？？？？？，解锁弹金色奖杯通知
- **DDL 提醒**：截止日期 5 级色阶着色 + 倒计时显示
- **天气**：当前天气 + 7 天预报（Open-Meteo，免费无 Key）
- **外观自定义**：6 套配色主题、透明度、字体、窗口位置大小记忆
- **桌面级驻留**：开机自启静默驻留托盘、抗 Win+D"显示桌面"隐藏
- **数据安全**：本地 SQLite 存储、日程导入 / 导出备份

## 技术栈

Tauri 2（Rust）+ Vue 3 + TypeScript + SQLite（rusqlite），构建为单文件便携 exe（静态链接 CRT）。

## 使用

Windows 10/11（需 WebView2 运行时，系统一般自带）。克隆后：

```bash
cd desktop-schedule
npm install
npm run tauri dev      # 开发
npm run tauri build -- --no-bundle   # 构建便携 exe
```

---

