PC 使用的图像工具包

最后会被构建为可执行二进制进行分发，而非直接引入代码。
避免 v8 的内存屏障导致应用崩溃。

在 macOS 上，它首选使用 `sips` 进行转换，其次才会使用 `image` 进行转码。

## CI 与发布流程

仓库使用 [GitHub Actions](https://github.com/features/actions)，工作流定义见 [`.github/workflows/ci.yml`](.github/workflows/ci.yml)。

### 触发条件

| 事件 | 运行内容 |
|------|----------|
| **Pull Request** | 在 `ubuntu-latest`、`macos-latest`、`windows-latest` 上执行 `cargo test` |
| **推送到 `main` / `master`** | 上述测试通过后，构建三平台 **release** 二进制，并作为 **Workflow Artifacts** 上传到该次运行结果页，便于下载 |
| **推送标签 `v*`**（如 `v0.1.0`） | 测试与构建通过后，使用 [softprops/action-gh-release](https://github.com/softprops/action-gh-release) **创建 GitHub Release**，并附带所有发布包 |
| **手动** | 可在 Actions 中选择本工作流，使用 **Run workflow**（`workflow_dispatch`）触发（仍遵循「非 PR 才构建 / 仅 `v*` 标签才发 Release」等条件） |

PR 仅跑测试，不执行 release 构建，避免未合并代码产出制品。

### 任务顺序

1. **`test`**：多平台单元测试与集成测试必须通过（`fail-fast: false`，任一平台失败可在日志中单独查看）。
2. **`build-release`**（仅非 PR 的 push）：在全部 `test` 任务成功后执行 `cargo build --release`，目标为：
   - `aarch64-apple-darwin`（macOS ARM）
   - `x86_64-apple-darwin`（macOS x86，在 Apple 芯片 Runner 上交叉编译）
   - `x86_64-pc-windows-msvc`（Windows x86_64）
3. **`publish-release`**：仅在 **push 且 ref 为 `refs/tags/v…`** 时运行；在 `build-release` 成功后下载 Artifacts，并发布到对应 **GitHub Release**（开启 `generate_release_notes`）。

### 发布包命名

| 平台 | 归档文件 | 说明 |
|------|----------|------|
| macOS ARM64 | `img-kit-aarch64-apple-darwin.tar.gz` | 解压后可执行文件名为 `img-kit-aarch64-apple-darwin` |
| macOS x86_64 | `img-kit-x86_64-apple-darwin.tar.gz` | 解压后可执行文件名为 `img-kit-x86_64-apple-darwin` |
| Windows x86_64 | `img-kit-x86_64-pc-windows-msvc.zip` | 内含 `img-kit-x86_64-pc-windows-msvc.exe` |

### 如何发版（创建 GitHub Release）

1. 确保默认分支（如 `main`）上的代码已通过 CI。
2. 使用语义化版本打标签并推送（示例）：

```bash
git tag -a v0.1.0 -m "v0.1.0"
git push origin v0.1.0
```

3. 在仓库的 **Releases** 页面查看自动创建的 Release 与附件；若需调整说明，可编辑 Release 正文。

仅推送到 `main` 而不打标签时，可在对应 Workflow 运行页的 **Artifacts** 中下载同架构的构建产物，但不会创建 Release。