# B4S：适用于 Windows、macOS 和 Linux 的非官方 Baseus 耳机应用

[English](README.md) | [Tiếng Việt](README.vi.md) | [Español](README.es.md) | 简体中文 | [Português (Brasil)](README.pt-BR.md)

**B4S 是一款免费开源的桌面应用，可在电脑上控制 Baseus（倍思）蓝牙 LE 耳机。**
无需手机上的 Baseus 应用，即可切换降噪（ANC）、通透和自适应模式，调节均衡器，
开启空间音频和游戏模式，并查看电量。支持 **Baseus Bass BP1 Pro** 和
**BP1 Ultra**，并为 **EP10 Ultra、EP10 Pro、Bowie M4s、Bowie MS1 和 Bowie M3s**
提供实验性配置。应用使用 SolidJS、Tauri 和 Rust 构建。

[**下载最新安装包**](https://github.com/hoan02/b4s/releases/latest) · [支持的耳机](#支持的-baseus-耳机) · [常见问题](#常见问题)

| 扫描并连接 | 设备控制 | 更多控制 | 设置 |
|---|---|---|---|
| ![B4S 在电脑上通过蓝牙 LE 扫描 Baseus 耳机](assets/i1.png) | ![B4S 显示 Baseus Bass BP1 Pro 的电量、降噪模式和空间音频](assets/i2.png) | ![B4S 将控制按声音、控制和设备分组](assets/i4.png) | ![B4S 设置：语言、主题和更新](assets/i3.png) |

截图展示的是越南语界面。控制界面由应用自身组件使用示例数据渲染。

## 0.1.3 新增内容

目录包含 124 个耳机配置，其中 122 个型号仅支持识别。产品名称和图片使用公开元数据；缩略图和大图保存在本地缓存中（64 MiB），可离线重复使用。型号 ID 已统一，已保存的设备和 EQ 数据会自动迁移。

由于更新签名密钥已更换，0.1.1/0.1.2 用户需要手动安装 0.1.3 一次。

[Changelog](CHANGELOG.md) · [Model contract](docs/model-identity-presentation.md) · [Image cache](docs/product-image-cache.md)

## 支持的 Baseus 耳机

B4S 只为已有审查配置的耳机启用控制。仅凭名称匹配不会开启任何控制功能。

| Baseus 型号 | 支持级别 | 可控制的功能 |
|---|---|---|
| Bass BP1 Pro | 已审查配置 | ANC、通透和自适应模式，EQ 预设和自定义 EQ，空间音频，游戏模式，Bass Boost，查找耳机 |
| Bass BP1 Ultra | 实验性，已在 Windows 上测试 | 电量、ANC、游戏模式、空间音频、Bass Boost、LDAC、听力保护、触控手势。EQ/SoundFit 不可用 |
| Bass EP10 Ultra、Bowie M4s、Bowie MS1 | 实验性，未在硬件上测试 | 与 BP1 Ultra 相同（共用适配器） |
| Bass EP10 Pro、Bowie M3s | 实验性，未在硬件上测试 | ANC、空间音频、游戏模式、Bass Boost、手势、风噪抑制（EP10 Pro 另有 EQ 预设） |
| 目录中其余 117 个 Baseus 型号 | 仅识别 | 在设备列表中显示名称和图片，无控制功能 |

部分控制（触控手势、入耳检测、多点连接、风噪抑制、左右自适应、恢复默认设置）
在应用中标记为**实验性**，并受 *Settings → Experimental mode* 控制。功能因型号
和固件而异。

### 支持级别

| 支持级别 | 含义 |
|---|---|
| 已验证 | 已在真实硬件上检查命令和行为。 |
| 实验性 | 已有协议配置，但仍需测试具体型号或固件。 |
| 仅识别 | 应用可以识别设备，但尚未启用控制功能。 |

请查看[型号目录](docs/model-catalog.md)和[协议说明](docs/protocol/overview.md)。

## 功能与开发

在设备支持时，B4S 可以显示左右耳机和充电盒电量，并控制降噪、通透、
均衡器、空间音频、游戏模式和耳机定位。你可以在 **Settings** 中开启自动
重连，应用启动时会搜索一次上次使用的受支持耳机；默认关闭，搜索最长
12 秒。也可以设置登录时启动 B4S。关闭窗口会在系统托盘可用时隐藏 B4S；
选择托盘菜单中的 **Quit B4S** 退出。功能因型号和固件而异。

无法扫描或连接时，请参阅[桌面故障排除指南](docs/desktop-troubleshooting.md)。

开发需要 Bun 1.4.0、Rust stable、对应平台的 Tauri 依赖；测试设备功能
还需要蓝牙耳机：

```sh
bun install --frozen-lockfile
bun run tauri:dev
```

应用默认使用英语，并提供越南语、简体中文、西班牙语和巴西葡萄牙语。
可在 **Settings** 中切换语言；翻译已打包，可离线使用。请参阅[翻译指南](docs/translations.md)
参与翻译。

## 常见问题

**Baseus 有适用于 Windows 或电脑的应用吗？**
Baseus 官方应用面向手机。B4S 是独立的非官方桌面应用，可在 Windows、macOS 和
Linux 上提供上述耳机控制。目前仅在 Windows 上做过测试。

**可以在电脑上调整 Baseus 的降噪（ANC）或 EQ 吗？**
可以（限受支持的型号）：通过蓝牙 LE 连接后，在 B4S 中切换 ANC、通透或自适应模式，
并选择 EQ 预设。

**我的 Baseus 耳机能用吗？**
请查看[支持的 Baseus 耳机](#支持的-baseus-耳机)。B4S 可识别 124 个 Baseus 耳机型号，
但只有该表中的型号才有控制功能。

**为什么 B4S 在 Windows 上找不到或无法控制我的耳机？**
Windows 常会把同一副耳机列出两次（音频端点和 BLE 控制入口）。请选择标有
**Control** 的条目。请参阅[故障排除指南](docs/desktop-troubleshooting.md)。

**安全吗？会把我的数据发送出去吗？**
控制通过蓝牙在本地运行。B4S 无需账号，也不会向服务器发送设备或个人数据。
请阅读[安全与许可](#安全与许可)。

## 文档

- [贡献指南](CONTRIBUTING.md)
- [架构](docs/architecture.md)
- [添加型号](docs/model-catalog.md)
- [发布与自动更新](docs/release.md)

## 安全与许可

B4S 是独立软件，未与 Baseus 或任何耳机制造商建立官方关联。设备控制
通过蓝牙在本地运行。定位耳机功能可能播放较大音量的声音，请先摘下耳机。
软件按现状提供，不保证兼容性、持续运行或固件恢复能力。

请勿将官方 APK、私钥、固件或受版权保护的反编译代码提交到仓库。许可证：MIT。
