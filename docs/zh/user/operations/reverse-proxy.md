# 反向代理示例（Nginx/Caddy）

Bastion 自身可以同时提供 HTTP 与 WebSocket。对外提供服务时，建议放在终止 TLS 的反向代理后面，并转发 `X-Forwarded-*` 头。

## 注意事项

- Bastion 默认会对非 loopback 流量强制 HTTPS。反向代理在 TLS 终止后，必须设置 `X-Forwarded-Proto: https`。
- 如果反向代理不在同一台机器上，需配置 trusted proxies：
  - `--trusted-proxy <proxy-ip>/32`（可重复指定）
  - 或 `BASTION_TRUSTED_PROXIES=10.0.0.10/32,10.0.0.0/24`
- WebSocket 端点（需要允许 upgrade）：
  - `/agent/ws`（Agent <-> Hub）
  - `/api/runs/<id>/events/ws`（run 的实时事件）

## Nginx（TLS 终止）

```nginx
map $http_upgrade $connection_upgrade {
  default upgrade;
  ''      close;
}

server {
  listen 443 ssl http2;
  server_name bastion.example.com;

  ssl_certificate     /etc/letsencrypt/live/bastion.example.com/fullchain.pem;
  ssl_certificate_key /etc/letsencrypt/live/bastion.example.com/privkey.pem;

  client_max_body_size 0;

  location / {
    proxy_pass http://127.0.0.1:9876;
    proxy_http_version 1.1;

    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;

    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection $connection_upgrade;
  }
}
```

## Caddy（TLS 终止）

```caddyfile
bastion.example.com {
  reverse_proxy 127.0.0.1:9876 {
    header_up Host {host}
    header_up X-Real-IP {remote_host}
    header_up X-Forwarded-For {remote_host}
    header_up X-Forwarded-Proto {scheme}
  }
}
```

## 纯 HTTP（仅限 LAN/开发环境）

在 LAN/开发环境（不启用 TLS）下，可以使用 `--insecure-http` / `BASTION_INSECURE_HTTP=1`，并绑定合适的 host/port：

```bash
BASTION_HOST=0.0.0.0 BASTION_PORT=9876 BASTION_INSECURE_HTTP=1 ./bastion
```

