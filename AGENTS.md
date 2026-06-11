# AGENTS

## changelog

### v2026.6.12 — 2026-06-12

- docs: 创建 opencode skill `opencode/skills/coreutils/SKILL.md`（完整命令参考 + 冲突表 + TTY 差异）— [12aa911]
- docs: 创建 `opencode/docs/coreutils.md`（instruction 源文件）— [12aa911]
- docs: 创建 `INSTALL_OPENCODE.md`（用户安装指南）— [12aa911]
- docs: `coreutils.exe --list` 增加分类标签 `[microsoft]`、`[uutils]`、`[custom]` — [fceba29]
- feat: 添加 `la` 作为 `ls -A` 的别名 — [fceba29]

### v2026.6.10 — 2026-06-10

- feat: 新建 AGENTS.md 规范文档（changelog/taolun/agents/项目进度/认知修正）— [fceba29]
- chore: 删除 `skills/` 旧目录（内容已迁移至 `opencode/`）— [12aa911]

### v2026.6.6 — 2026-06-10

- feat: `coreutils.exe --list` 输出增加分类标签 `[microsoft]`、`[uutils]`、`[custom]` — [8b7111a]
- feat: 添加 `la` 作为 `ls -A` 的别名 — [8b7111a]
- chore: 抑制 winfo/dfree FFI 警告 — [83cad75]
- fix: 对齐 winfo 和 dfree 中 CreateFileW/DeviceIoControl 的 FFI 签名 — [83cad75]
- feat: 合并 dfree TUI 磁盘分析器 — [23fe43f]
- fix: winfo IOCTL 磁盘类型检测 + SMBIOS 内存频率 — [dc1f789]
- fix: wtop 参数解析跳过首个工具名参数 — [dbe4d2b]
- feat: 添加 `which` 命令 — [2c62aba]

[8b7111a]: https://github.com/Titor-Z/coreutils/commit/8b7111a
[83cad75]: https://github.com/Titor-Z/coreutils/commit/83cad75
[9faa941]: https://github.com/Titor-Z/coreutils/commit/9faa941
[23fe43f]: https://github.com/Titor-Z/coreutils/commit/23fe43f
[dc1f789]: https://github.com/Titor-Z/coreutils/commit/dc1f789
[2c62aba]: https://github.com/Titor-Z/coreutils/commit/2c62aba
[dbe4d2b]: https://github.com/Titor-Z/coreutils/commit/dbe4d2b

### v2026.6.5 — 2026-06-09

- feat: 集成 wtop TUI 进程监控工具
- feat: 添加 winfo 系统信息命令（neofetch-like）

### v2026.6.4 — 2026-06-08

- feat: 添加 GitHub Actions 构建和发布工作流
- chore: vendor diffutils/tar 源码（代替子模块）
- docs: 更新 README，说明自定义 fork 差异

---

## taolun

### 2026-06-10：项目对比与 --list 增强

**用户需求**：`coreutils.exe --list` 显示的列表不完整，需要对比 Microsoft/coreutils 的差异。

**分析发现**：
1. 项目有三层结构：Microsoft fork（deps/coreutils）、额外 uutils 生态包（findutils/grep/sed/diffutils/tar）、自研模块（dfree/winfo/wtop/which）
2. 子模块未初始化（HTTPS 连不上 GitHub），改用 SSH 协议后拉取成功
3. `la` 别名缺失（Microsoft 文档有，我们的 --list 没有）
4. `which` 虽然代码齐全（crates/uu_which/），但 build.rs 能自动发现，无需手动注册

**解决方案**：
1. build.rs 添加 `la` 别名映射 → `(ls::uumain, ls::uu_app)`
2. main.rs 添加 `"la" => "ls"` 规范名映射
3. main.rs 添加 `command_category()` 函数，按来源标记三类命令
4. `--list` 输出改为 `命令名            [标签]` 格式

**命令分类结果**：
- `[microsoft]` (71个)：来自 deps/coreutils/src/uu/*
- `[uutils]` (7个)：cmp, diff, find, grep, sed, tar, xargs
- `[custom]` (5个)：dfree, la, which, winfo, wtop

**后续**：创建 AGENTS.md，规范开发流程。

### 2026-06-12：创建 opencode skill 与项目对比文档

**用户需求**：需要维护一个 coreutils 的 opencode skill，包含命令参考、冲突规则、TTY 差异。

**分析发现**：
1. opencode skill 规范路径为 `.opencode/skills/<name>/SKILL.md`，需 YAML frontmatter（name + description）
2. 已有 `~/.config/opencode/docs/coreutils.md` 作为 instruction 自动加载，但内容过简
3. 最佳方案：instruction 只放必须遵守的核心规则，skill 放完整参考手册

**实施**：
1. 创建 `.opencode/skills/coreutils/SKILL.md`（200+ 行完整参考）
2. 精简 `~/.config/opencode/docs/coreutils.md` 为使用规范（冲突表 + 禁止规则 + TTY 差异）
3. instruction 末尾加链接指向 skill，方便 opencode 按需加载

**后续**：release 构建交给 GitHub Actions 自动处理。

### 2026-06-12：opencode skill 作为交付物

**用户需求**：重构文件结构，使 skill/instruction 能从 repo 直接复制到用户 `~/.config/opencode/`。

**分析发现**：
1. `.opencode/skills/coreutils/SKILL.md` 是项目级 skill，不会暴露给用户在其它项目使用
2. 用户需要的是 `~/.config/opencode/skills/coreutils/SKILL.md`（全局 skill）和 `~/.config/opencode/docs/coreutils.md`（全局 instruction）
3. 最佳交付方案：repo 中放一份在 `opencode/` 目录下，附 INSTALL_OPENCODE.md 说明复制步骤

**实施**：
1. 创建 `opencode/skills/coreutils/SKILL.md`（交付用 skill，与旧 `.opencode/` 版内容相同）
2. 创建 `opencode/docs/coreutils.md`（交付用 instruction，更新了链接指向新 skill 路径）
3. 创建 `INSTALL_OPENCODE.md`（用户安装指南，三步复制 + 配置 opencode.jsonc）
4. 删除 `.opencode/skills/coreutils/`（项目级 skill 不再需要）

**结论**：三份交付文件均就绪，用户只需从 GitHub 检出项目后按 INSTALL_OPENCODE.md 操作即可。

---

## Agents 规范

### 1. 强制停止规则

一个问题重复 **3 次** 无法解决完成，**强制停止**，向用户详细汇报遇到的问题，等待用户的解答。

### 2. 语言要求

整个对话流程中，全部强制使用中文，包括 AI 思考过程（thought）打印在终端中的内容。

### 3. 注释规范

项目必须有详细的**中文注释**，包括：
- 模块/结构体/函数的用途说明
- 复杂逻辑的关键步骤解释
- FFI 外部函数接口的参数说明
- 临时 `#![allow]` / `#[allow]` 的原因标注

### 4. 版本发布格式

```
YYYY.MM.DD.xxxx
```

其中 `xxxx` 为作为 git tag 前的 commit ID（前 4 位），方便溯源。

示例：`2026.6.10.a1b2`

### 5. 测试文件规范

测试文件应该按照功能模块拆分成多个文件，**禁止在一个文件里写全部测试内容**。

### 6. 开发模式

采用 **OOP 面向对象方式**，保持功能模块的单一，做到**高内聚低耦合**。

### 7. Shell 使用规范

开发时，Windows 系统已内置 coreutils 组件，可以像在 Linux 上一样使用 bash 命令：
```
grep, ls, seq, sed, find, sleep, head, tail, sort, wc, cat
```
而无需使用 PowerShell cmdlet（如 `Get-ChildItem`、`Select-String`、`Start-Sleep` 等）。

### 8. 开发流程

1. **先保存讨论记录**，然后开始改动文件内容
2. 将讨论摘要写入 `taolun` 章节
3. 开发完成后，更新：
   - `项目进度` 章节
   - `changelog` 章节
4. changelog 的内容要和 taolun 记录、项目进度里的栏目**形成外链**，方便后期溯源

---

## 项目进度

### 计划中

- GitHub Actions 自动构建 release 版本
- 完整的测试覆盖（按模块拆分）

### 代办

- (暂无)

### 已完成

- [x] `which` 命令（PATH 查找，PATHEXT 支持，`-a` 参数）— [v2026.6.5]
- [x] `winfo` 系统信息 TUI（ratatui，莫兰迪配色，SMBIOS/BIOS/CPU/内存/NIC）— [v2026.6.5]
- [x] `wtop` 进程监视器 TUI — [v2026.6.5]
- [x] `dfree` 磁盘分析 TUI（ratatui，磁盘分组/文件分类/大文件浏览/toml 配置）— [v2026.6.6]
- [x] `la` 别名（`ls -A`）— [v2026.6.6]
- [x] `--list` 输出增加分类标签（microsoft / uutils / custom）— [v2026.6.6]
- [x] 子模块拉取（SSH 代替 HTTPS）— [v2026.6.6]
- [x] 创建 AGENTS.md 规范文档 — [fceba29]
- [x] dfree.toml 配置文件支持
- [x] opencode skill 文档（`opencode/skills/coreutils/SKILL.md`）— [12aa911]
- [x] opencode instruction 交付文件（`opencode/docs/coreutils.md`）— [12aa911]
- [x] 用户安装指南（`INSTALL_OPENCODE.md`）— [12aa911]

[v2026.6.5]: https://github.com/Titor-Z/coreutils/tree/v2026.6.5
[v2026.6.6]: https://github.com/Titor-Z/coreutils/tree/v2026.6.6

---

## 认知修正

### 2026-06-10

1. **子模块 URL 协议**：GitHub HTTPS 连接超时，改用 SSH（`git@github.com:...`）后成功。.gitmodules 中的 URL 需要手动修改。
2. **ntfind 不是单独命令**：`ntfind` 是通过 `find_uumain` 包装整合进 `find` 命令内部的，不在 `--list` 中独立显示。
3. **build.rs 自动发现机制**：`uu_*` 前缀的包会被自动加入 map，无需在 build.rs 中硬编码（但 `which` 是个例外，因为路径不在 `deps/coreutils/` 内）。
4. **LTO 编译极慢**：release 构建启用 fat LTO + codegen-units=1，单次完整编译约 17 分钟。debug 构建约 3-4 分钟。日常开发建议用 `cargo check` 或 debug 构建。
5. **`sort` 在 build.rs 中是特例**：虽然 `sort = { package = "uu_sort" }` 在 Cargo.toml 中，但 build.rs 用 `has_sort` 特殊处理了它（因为需要 ntsort 包装）。它不在 `coreutils` 自动发现列表中。
6. **`grep` 也在自动发现列表中**：`grep = { package = "uu_grep", path = "deps/grep" }` 虽然路径在外，但 build.rs 的 `strip_prefix("uu_")` 逻辑会将 `uu_grep` 识别并加入 `coreutils` 列表。因此 grep 的表现和其他 `uu_*` 包一致。
7. **已经写了 `#![allow]` 的模块不能再写一次**：在 `src/dfree/mod.rs` 中，模块级 `#![allow(non_snake_case, non_camel_case_types, unused)]` 应该只出现一次，放在文件最顶部。
