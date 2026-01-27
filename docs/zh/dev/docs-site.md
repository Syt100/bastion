# 文档站（VitePress + 内置到产品）

Bastion 使用 **VitePress** 构建文档站。

## 源码与输出目录

- 源码：`docs/`
- 构建输出：`docs/.vitepress/dist/`

## Base path（`/docs/` vs GitHub Pages）

文档站的 base 可通过 `DOCS_BASE` 覆盖。

当文档作为产品内置（由 Hub 提供 `/docs/`）时，建议用：

```bash
DOCS_BASE=/docs/ npm run build --prefix docs
```

## Hub 提供文档服务

Hub 会在以下路径提供文档：

- `/docs/`（默认公开）

模式：

- **Filesystem mode**（默认开发态）：从 `docs/.vitepress/dist/` 读取静态文件
  - 可用 `BASTION_DOCS_DIR=/path/to/dist` 覆盖
- **Embedded mode**：通过 `--features embed-docs`（或 `embed-web`）把 docs 编译进二进制
  - 需要在编译前准备好 `docs/.vitepress/dist/`

## 与 UI 的集成

Web UI 提供 **Help** 入口，会在新标签页打开 `/docs/`。

当你在开发 UI（`npm run dev --prefix ui`）时，UI dev server 会把 `/docs/*` 代理到 Hub，这样 Help 链接仍可用。

