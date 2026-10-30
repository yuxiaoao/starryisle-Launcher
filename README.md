<div align="center">
  <img src="./apps/app/icons/128x128.png" width="128" height="128" alt="StarryIsle Launcher Logo" />
  <h1>StarryIsle Launcher</h1>
  <p><strong>基于 Axolotl Launcher 二次开发的 Minecraft Java 版第三方启动器。</strong></p>
</div>

> 本项目是 [Axolotl Launcher](https://github.com/Mystic-Stars/Axolotl) 的个人二次开发版本，遵循 GPL-3.0-only 继续开源。
> 与原项目、Rinth, Inc.（Modrinth）及 Mojang 均无隶属或背书关系。

## 这是什么

StarryIsle 在原版基础上改动品牌与系统标识（应用名、图标、bundle identifier、deep-link 协议），
功能继承自上游：Modrinth / CurseForge 内容浏览与安装、实例管理、多种账户登录、实验室工具（种子地图、配方生成器、皮肤编辑器）等。

- 应用标识：`com.starryisle.launcher`
- 链接协议：`starryisle://`
- 数据目录：`%APPDATA%\com.starryisle.launcher`（macOS/Linux 为对应配置目录）

## 截图中心：时间线

截图中心是本分支重点改造的部分，核心是一个按时间轴排列的**时间线视图**。

在截图页顶部的「视图」下拉里可以在「网格」和「时间线」之间切换：

- **按天分组**：每天一个日期标题，右侧显示当天截图数量。
- **整点分隔**：同一天内截图较多时，按小时插入分隔节点，方便回看某个时段的进度。
- **虚拟滚动**：只渲染可见的几行，几千张截图也能流畅滚动，鼠标停在卡片上用滚轮即可翻页。
- **缩略图预览**：列表读取 512px 缩略图而不是几 MB 的原图，打开页面几乎瞬间完成。
- **按实例 / 分组筛选**：时间线默认展示全部实例，可以只留某个整合包，或只看某个自定义分组。

每张卡片显示截图时间、所属实例、文件名；如果写过备注，右侧会多出一列备注内容，中间用一条竖线分隔。

### 截图备注

在时间线或网格里右键任意截图 →「编辑备注」，即可就地写下一句不超过 30 字的说明，回车保存、Esc 取消。

- 备注存在本地数据库的独立表里，按「实例 + 文件名」关联，文件内容变化后备注依然保留。
- 搜索框同时匹配**文件名、实例名和备注**，所以可以用备注找回某张截图。
- 删除截图时会一并清理对应备注，不会留下孤儿数据。

### 截图包导出 / 导入

选中截图后，底部操作条上的图标按钮可以把它们打包带走：

- **导出包**：把选中的截图、对应备注和一个清单文件打成一个 zip。
- **导入包**：在另一台电脑上选回这个 zip，图片和备注会一起还原，可选择「跳过 / 覆盖 / 保留两者」。
- 解压时会校验每个条目的路径，防止恶意压缩包把文件写到目标目录之外。

备注按文件名关联，所以换机器、换实例路径都能对上；配合时间线，相当于一份带注解的游戏历程。

## 从源码构建

需要 Node.js 24.15、pnpm 10.33、Rust 1.95、Tauri v2 系统依赖；Windows 上还需要 MSVC 生成工具、
MinGW gcc（编译种子地图原生模块）与 JDK 17（编译内置 Java 组件）。

```bash
git clone --recursive https://github.com/yuxiaoao/starryisle-Launcher.git
cd starryisle-Launcher
corepack enable
pnpm install --frozen-lockfile
pnpm app:dev
```

打包与发布流程沿用上游：打 tag 触发 `.github/workflows/axolotl-release.yml`。
发布前请先配置自己的 Tauri 签名密钥与更新源，并运行 `pnpm axolotl:brand-guard` 检查品牌一致性。

> 本地开发请设置 `THESEUS_CONFIG_DIR`，避免与已安装版本共用数据库（迁移是单向的）。

## 许可与署名

- 本仓库整体按 **GPL-3.0-only** 分发，各子包的许可见对应目录下的 `COPYING.md` / `LICENSE`。
- 原始作品版权归 Rinth, Inc.、Garbage Human Studio 及 Axolotl 贡献者所有；本二次开发版本的修改部分由
  本仓库维护者保留相应权利，并同样以 GPL-3.0-only 授权。
- Modrinth 名称与图形标识归 Rinth, Inc. 所有，本项目仅在描述兼容性时提及，不使用其品牌素材。
- Minecraft 是 Mojang Synergies AB 的商标；本项目未获得 Mojang 或 Microsoft 的授权或背书。
- 第三方组件（Blockbench、cubiomes、Deepslate 等）的许可见 `third-party/licenses/`。

## 上游

- Axolotl Launcher：https://github.com/Mystic-Stars/Axolotl
- 上游基于 Modrinth App：https://github.com/modrinth/code
