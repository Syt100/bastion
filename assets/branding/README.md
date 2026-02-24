# Bastion Brand Icon Assets

This directory contains generated icon assets for Bastion.
All SVG/PNG/ICO outputs are produced from one shared SVG source in
`scripts/generate-brand-icons.py` so every format stays visually aligned.

- `icon-master.svg`: vector master output (shield + sync ring)
- `icon-master-1024.png`: high-resolution raster master
- `icon-*.png`: exported sizes for UI/docs/app stores
- `bastion.ico`: Windows multi-size icon bundle

## Regenerate

```bash
python3 -m pip install --target /tmp/pylib pillow resvg
PYTHONPATH=/tmp/pylib python3 scripts/generate-brand-icons.py
```

The generator also updates:

- `ui/public/favicon.ico`
- `ui/public/favicon-16x16.png`
- `ui/public/favicon-32x32.png`
