# ffg go 多版本管理工具

[en](./README.md)

## 安装

```bash
cargo install ffg
```

## 环境变量设置

- `FFG_HOME` 默认值为 `~/.ffg`
- `FFG_MIRROR` 默认值为 [go.dev](https://go.dev), 境内可以设置为 `https://studygolang.com`

## 使用

- `ffg ls` 枚举已经安装版本
- `ffg ls-remote` 获取可用的版本
- `ffg use 1.15.6` 安装并使用指定版本
- `ffg rm 1.15.6` 删除指定版本

## 添加 goroot 到 path

```bash
# 设置 ffg 跟目录
export FFG_HOME=~/.ffg
# 将 goroot 添加到环境变量
export PATH=$PATH:$FFG_HOME/go
```
