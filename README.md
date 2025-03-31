# Kidfile & Kidfile Explorer

![image](https://github.com/user-attachments/assets/1f161034-366d-4f39-af5f-45bb904757b0)

[![Build](https://github.com/malucard/kidfile/actions/workflows/build.yml/badge.svg)](https://github.com/malucard/kidfile/actions/workflows/build.yml) [![GitHub Release](https://img.shields.io/github/v/release/malucard/kidfile)](https://github.com/malucard/kidfile/releases/latest)

This is an attempt to collect code to decode all file formats in the Infinity series in one accessible place.
Kidfile is a library that can be used by other projects to handle supported files easily.
Kidfile Explorer is an application that can navigate, preview and convert those files in batch.

Currently supports images in most of the PS2 games and their late PSP and PC ports.

- Image formats:
  - OGDT
  - TIM2
  - BIP (all)
  - GIM (PSP)
  - KLZ
  - PVR (Dreamcast)
  - common formats (PNG, BMP, etc)
- Archive formats:
  - AFS
  - LNK (partial)
  - Concatenated OGDT/TIM2 images
- Compression formats:
  - LZSS
  - Big-endian LZSS-like used in N7 DC and 12R PS2
  - CPS (PS2)
