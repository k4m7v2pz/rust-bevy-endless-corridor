# Reference Projects

此目录用于存放**各类仓库**（自己的或他人的），作为本项目的参考资料。

## 使用方式

在 `reference/` 目录下执行 `git clone` 来添加参考仓库：

```bash
cd reference
git clone <另一个仓库的URL>
```

## 目录结构

```
reference/
├── README.md             # ✅ 提交（说明文档+保留目录）
└── <其他仓库>/           # ❌ 不提交（通过 git clone 添加）
```

## 注意事项

- ⚠️ 所有子目录都会被 `.gitignore` 忽略，不会提交到 Git
- ✅ `README.md` 会保留在仓库中
- 💡 每次使用 `git clone` 后，这些仓库不会影响主项目的 Git 状态

### .gitignore 配置

项目根目录的 `.gitignore` 中关于 `reference/` 的配置：

```gitignore
reference/*
!reference/README.md
```

## License Attributions

引用仓库的许可证声明位于各子目录的 `LICENSE` 或 `LICENSE.txt` 中。
