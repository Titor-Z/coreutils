# 安装 opencode Skill 和 Instruction

为 opencode 安装 coreutils skill/instruction 后，opencode 在任何项目中都能正确使用 coreutils 命令。

---

## 文件清单

| 文件 | 用途 |
|------|------|
| `opencode/skills/coreutils/SKILL.md` | opencode skill（按需加载的完整命令参考） |
| `opencode/docs/coreutils.md` | opencode instruction（自动加载的核心使用规则） |

## 安装步骤

### 第 1 步：复制 skill

```powershell
# 从项目根目录执行
mkdir -Force ~\.config\opencode\skills\coreutils
copy opencode\skills\coreutils\SKILL.md ~\.config\opencode\skills\coreutils\SKILL.md
```

### 第 2 步：复制 instruction

```powershell
mkdir -Force ~\.config\opencode\docs
copy opencode\docs\coreutils.md ~\.config\opencode\docs\coreutils.md
```

### 第 3 步：配置 opencode.jsonc

编辑 `~\.config\opencode\opencode.jsonc`，添加 instruction 引用：

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "instructions": [
    "~/.config/opencode/docs/coreutils.md"
  ]
}
```

如果已有其他 instruction，追加到数组中即可：

```jsonc
{
  "instructions": [
    "~/.config/opencode/docs/some-other.md",
    "~/.config/opencode/docs/coreutils.md"
  ]
}
```

---

## 验证安装

1. 启动 opencode
2. 确认 `available_skills` 中包含 `coreutils` skill
3. 测试指令是否生效：

```
ls.exe -la
cat.exe --help
```

如果看到正确的输出，说明安装成功。

---

## 卸载

```powershell
remove ~\.config\opencode\skills\coreutils\SKILL.md
remove ~\.config\opencode\docs\coreutils.md
```

然后从 `opencode.jsonc` 中移除相应的 `"~/.config/opencode/docs/coreutils.md"` 行。

---

## 更新

coreutils 发布新版时，重新执行安装步骤即可覆盖旧文件。
