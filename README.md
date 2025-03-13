# Kidfile & Kidfile Explorer

![image](https://github.com/user-attachments/assets/1f161034-366d-4f39-af5f-45bb904757b0)

[![Build](https://github.com/malucard/kidfile/actions/workflows/build.yml/badge.svg)](https://github.com/malucard/kidfile/actions/workflows/build.yml) [![GitHub Release](https://img.shields.io/github/v/release/malucard/kidfile)](github.com/malucard/kidfile/releases/latest)

This is an attempt to collect code to decode all file formats in the Infinity series in one accessible place.
Kidfile is a library that can be used by other projects to handle supported files easily.
Kidfile Explorer is an application that can navigate, preview and convert those files in batch.

Currently supports all images from Never7 on PS2 and most images from 12Riven on PC. Lots of other formats used in the other games and releases are missing, but it's not hard to add support for each.

- Archive formats:
  - AFS
  - LNK
  - Concatenated files aligned to 2KiB
- Compression formats:
  - LZSS/BIP
  - CPS (PS2)
- Image formats:
  - OGDT
  - TIM2
  - common formats (PNG, BMP, etc)
