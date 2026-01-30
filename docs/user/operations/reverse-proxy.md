# Reverse proxy (Nginx/Caddy) examples

Bastion itself can serve HTTP and WebSocket traffic. For public access, deploy it behind a reverse proxy that terminates TLS and forwards `X-Forwarded-*` headers.

## Notes

- Bastion enforces HTTPS by default for non-loopback traffic. If you terminate TLS at the proxy, it MUST set `X-Forwarded-Proto: https`.
- If your reverse proxy is not on the same host as Bastion, configure trusted proxies:
  - `--trusted-proxy <proxy-ip>/32` (repeatable)
  - or `BASTION_TRUSTED_PROXIES=10.0.0.10/32,10.0.0.0/24`
- WebSocket endpoints (must allow upgrade):
  - `/agent/ws` (Agent <-> Hub)
  - `/api/runs/<id>/events/ws` (live run events)
- If you rely on automatic language selection for `/docs`, ensure the proxy forwards `Accept-Language` and `Cookie` headers (most proxies do by default).

## Nginx (TLS termination)

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

## Caddy (TLS termination)

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

## Plain HTTP (LAN/dev only)

For LAN/dev (no TLS), run Bastion with `--insecure-http` or `BASTION_INSECURE_HTTP=1` and bind a suitable host/port:

```bash
BASTION_HOST=0.0.0.0 BASTION_PORT=9876 BASTION_INSECURE_HTTP=1 ./bastion
```
