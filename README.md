# ffg go mutil version manager

[中文](./README_ZH.md)

## install

```bash
cargo install ffg
```

## set environment

- `FFG_HOME` default value is `~/.ffg`
- `FFG_MIRROR` default value is [go.dev](https://go.dev), if work in china's mainland please set this value `https://studygolang.com`

## usage

- `ffg ls` list local version
- `ffg ls-remote` pull release version
- `ffg use 1.15.6` install and used
- `ffg rm 1.15.6` remove
- `ffg ins 1.15.6` just install the specify version

## setting go path

```bash
# set ffg home
export FFG_HOME=~/.ffg
# add goroot to path
export PATH=$PATH:$FFG_HOME/go/bin
```
