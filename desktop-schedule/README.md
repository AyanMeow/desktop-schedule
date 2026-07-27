# 桌面日程（desktop-schedule）

Windows 桌面日程管理小组件 —— 常驻桌面、半透明、无边框的轻量日程贴片。

技术栈：**Tauri 2.x（Rust）+ Vue 3 + TypeScript**。

## 开发环境

前置依赖（已在本机配置）：
- Microsoft C++ Build Tools 2022（VCTools 工作负载）— 提供 MSVC 编译器
- Rust stable-msvc 工具链（`rustup default stable-msvc`）
- Node.js LTS
- WebView2 运行时（Win10/11 默认预装）

本机已做的加速配置：
- `~/.cargo/config.toml` —— crates.io 走 rsproxy 镜像（sparse 协议）

## 常用命令

```bash
# 开发（热重载，首次编译约 2-3 分钟）
npm run tauri dev

# 类型检查 / Rust 编译检查
cd src-tauri && cargo check

# 生产构建（产出安装包）
npm run tauri build

# 便携 exe（不生成安装包，配合 +crt-static 可解压即用）
npm run tauri build -- --no-bundle
```

## 项目结构

```
desktop-schedule/
├── src/                     # Vue 前端
│   └── App.vue              # 主界面（M0：透明度/背景/锁定/置顶/自启控制台）
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs           # Tauri 主入口：托盘、自启、窗口命令
│   │   └── window_state.rs  # 窗口锁定状态
│   ├── capabilities/        # 权限配置（前端可调用的命令）
│   ├── .cargo/config.toml   # 静态 CRT 配置（便携 ZIP 用）
│   └── tauri.conf.json      # Tauri 配置（无边框透明窗口在此定义）
└── package.json
```

## 关键设计决策

- **窗口形态**：默认"普通贴片"——半透明、非置顶、正常 z-order（可被其他窗口遮挡）。非"壁纸层嵌入"，非"置顶"。
- **透明度**：Tauri 2 无窗口级 `set_alpha`，由前端 CSS `opacity` 控制。
- **托盘与贴片解耦**：关闭按钮 → 隐藏到托盘（不退出）；托盘菜单/左键可切换显示。
- **便携分发**：`+crt-static` 让 exe 不依赖 vcruntime；面向 Win10/11（已预装 WebView2）可直接打 ZIP。

## 推荐 IDE 配置

[VS Code](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

详见上层目录 `设计方案.md`。
