# Web UI 视觉规范（设计系统）

本文档是 Bastion Web UI 的“权威视觉规范”，面向贡献者与开发者。
当你新增/修改 UI 时，请以本规范为准，确保在不同页面、浅色/深色模式、以及主题预设之间保持一致。

## 目标

- 保持跨页面、跨功能、跨贡献者的视觉一致性。
- 让浅色/深色模式与主题预设在默认情况下始终正确。
- 让“一致的写法”成为最省事的写法：优先使用 tokens，其次复用统一的 recipes，最后才做局部微调。

## 真相源（Where the truth lives）

- 主题与 tokens：`ui/src/styles/main.css`
- Naive UI 主题集成：`ui/src/App.vue`
- 共享布局模式：`ui/src/layouts/AppShell.vue`、`ui/src/components/*`

如果需要新增颜色/阴影/表面层级等，请先考虑复用或补充 token；不要在页面里直接写死颜色。

## Token 模型

### Theme tokens（按主题预设）

主题预设通过给 `<html>` 设置 `data-theme`，并配合 `.dark`，在 CSS 变量层面定义浅色/深色两套值：

- 浅色：`[data-theme="..."] { ... }`
- 深色：`.dark[data-theme="..."] { ... }`

常用 token（不完全）：

- 主色/强调：`--app-primary`, `--app-primary-soft`, `--app-primary-2`
- 文本：`--app-text`, `--app-text-muted`
- 表面：`--app-bg-solid`, `--app-surface`, `--app-surface-2`, `--app-bg`（背景 aurora 层）
- 边框与交互状态：`--app-border`, `--app-hover`, `--app-pressed`
- 状态语义：`--app-info`, `--app-success`, `--app-warning`, `--app-danger`（全局语义）

### Foundation tokens（全局稳定）

Foundation tokens 不应随主题预设变化，用来定义产品级一致的“基础尺度”：

- 圆角：`--app-radius-sm`, `--app-radius`, `--app-radius-lg`
- 动效：`--app-duration-fast`, `--app-duration-normal`, `--app-ease-standard`

## Tailwind 使用规则（长期最佳实践）

### 推荐用法

- 使用 Tailwind 的尺度体系做布局/间距/字号/圆角（尽量避免 arbitrary values）：
  - `p-4`, `gap-3`, `text-sm`, `rounded-xl` 等。
- 颜色必须走 `var(--app-...)`：
  - `bg-[var(--app-surface-2)]`
  - `text-[var(--app-danger)]`
  - `border-[color:var(--app-border)]`

### 避免/禁止（除非有明确的、文档化的例外）

- 写死的调色板颜色：`text-red-*`, `bg-amber-*`, `border-slate-*`, `dark:text-blue-*` 等。
- 用透明度表达“次要文本”：`opacity-70` / `opacity-80`（应使用 `--app-text-muted`）。
- 不随主题变化的 chrome 颜色：`bg-white/..`, `border-black/..`, `divide-black/..` 等。

## 语义化工具类（Semantic utilities）

优先使用这些共享 class，避免重复造轮子：

- `app-text-muted`：次要/说明文本颜色。
- `app-border-subtle`：使用 `--app-border` 的 1px 细边框。
- `app-divide-y`：列表分割线（对子元素生效，使用 token）。
- `app-panel-inset`：内嵌面板（`--app-surface-2` + 细边框）。
- `app-card`：统一的 card 阴影层级。
- `app-list-row`：统一的可点击行（间距 + hover）。
- `app-mono-block`：用于 ID/路径/片段的等宽文本块。
- `app-kbd`：快捷键 keycap 样式。
- `app-glass`, `app-glass-soft`：用于导航 chrome 的玻璃态（谨慎使用）。

## 组件配方（Recipes）

### Card（内容容器）

默认内容卡：

```vue
<n-card class="app-card" :bordered="false">
  ...
</n-card>
```

规则：只要 `n-card` 上使用了 `class="app-card"`，就必须同时设置 `:bordered="false"`。卡片的层级感由 `app-card` 的阴影提供；如果有的卡带边框、有的卡不带边框，会让 UI 在长期演进中非常容易“漂移”。

卡片内的“内嵌面板”：

```vue
<div class="rounded app-panel-inset p-3">
  ...
</div>
```

### 次要文本（Muted text）

用 token 驱动的次要文本颜色：

```html
<div class="text-sm app-text-muted">说明文字…</div>
```

不要用透明度“伪造”次要文本：

```html
<!-- 不推荐 -->
<div class="text-sm opacity-70">说明文字…</div>
```

### 列表分割线

在列表容器上使用 `app-divide-y`：

```html
<div class="app-divide-y">
  <button class="app-list-row">...</button>
  <button class="app-list-row">...</button>
</div>
```

### 状态与错误

优先使用 Naive UI 的语义组件：

```vue
<n-alert type="error" :bordered="false">...</n-alert>
<n-tag size="small" :bordered="false" type="warning">...</n-tag>
```

如果必须用纯文本颜色，使用 token：

```html
<div class="text-xs text-[var(--app-danger)]">错误信息…</div>
```

### 表格

所有 `n-data-table` 应共享同一套视觉规则（表头、hover、选中态等），并尽量通过全局 class 在 `ui/src/styles/main.css` 中统一管理。

### 等宽文本与快捷键

```html
<div class="app-mono-block break-all">{{ id }}</div>
<kbd class="app-kbd">Ctrl</kbd>
```

## Code review 检查清单

- 语义颜色使用 `--app-*` tokens（不使用 Tailwind 颜色调色板）。
- 次要文本使用 `app-text-muted`（不使用 opacity）。
- 分割线/细边框使用 `app-divide-y` / `app-border-subtle`。
- 复用共享 recipes（`app-card`、`app-list-row`、toolbars 等），不要在页面里重复发明。
- 浅色/深色模式 + 主题预设下都正确。
