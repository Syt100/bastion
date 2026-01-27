# 构建与运行

## 前置条件

- Rust `1.92+`
- Node.js `20.19+` 或 `22.12+`

## 运行 Hub（后端）

```bash
cargo run -p bastion
```

默认监听：`127.0.0.1:9876`。

## 以 dev 模式运行 Web UI

终端 1：

```bash
cargo run -p bastion
```

终端 2：

```bash
npm ci --prefix ui
npm run dev --prefix ui
```

打开 `http://localhost:5173`。

UI dev server 会代理：

- `/api/*` → Hub
- `/agent/*` → Hub（WebSocket）
- `/docs/*` → Hub（内置文档）

## 以 dev 模式运行文档站

```bash
npm ci --prefix docs

# VitePress 默认使用 5173；如果 UI dev server 已在跑，请换一个端口。
npm run dev --prefix docs -- --port 5174
```

## 构建内嵌资源（embedded builds）

构建 UI：

```bash
npm ci --prefix ui
npm run build-only --prefix ui
```

构建 docs：

```bash
npm ci --prefix docs
DOCS_BASE=/docs/ npm run build --prefix docs
```

然后以嵌入资源的方式构建 Hub：

```bash
cargo build -p bastion --features embed-web
```

说明：

- `embed-web` = `embed-ui` + `embed-docs`
- 如果只想嵌入其中之一，可使用 `--features embed-ui` 或 `--features embed-docs`

