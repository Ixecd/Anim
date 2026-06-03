# ADR 008: FSIR 跨语言 ABI — Postcard 二进制格式

> 状态：已定稿
> 日期：2026-06-03
> 对应：FORGET P0 #9 — FSIR 跨语言 ABI 零设计

---

## 动机

FSIR 目前只有 JSON 序列化。双流水线架构要求 Go Server 在设备空闲时离线预交织 `.anim` 源码，生成 FSIR 缓存到设备本地。JSON 有两个问题：

1. **体积大**：带格式的 JSON 相比二进制大了 2-4 倍。设备启动时加载缓存——毫秒级延迟差异。
2. **哈希不稳定**：JSON 的空白和 key 顺序可能随序列化器版本微调而改变——同样的 FSIR 内容产生不同哈希，缓存失效逻辑被误触发。

需要一种二进制 wire format：体积小、哈希确定性、跨语言、`#[no_std]` 兼容。

---

## 决策

使用 **Postcard**（`postcard` crate）作为 FSIR 二进制格式。

### 为什么是 Postcard，不是 FlatBuffers 或 Bincode

```
FlatBuffers
    ✅ 零拷贝——直接从字节数组读字段值
    ❌ 需要 schema 编译器（flatc）→ 构建流程多一个外部工具
    ❌ Rust 端需要导入生成的代码——和手写的 FsirDoc 是两个源
    ❌ Go 端同样需要 schema 编译器
    → 过度设计。FSIR 的字段树只有 3 层——不需要零拷贝带来的复杂度。

Bincode
    ✅ Serde 原生支持，和 JSON 一样只需 derive
    ❌ bincode 1.x 格式有多个不兼容版本（Default/Compat/Varint）
    ❌ bincode 2.x 仍在 beta——API 频繁变动
    → 版本混乱。选型风险太大。

Postcard
    ✅ Serde 原生支持——只需 derive Serialize + Deserialize
    ✅ #[no_std] 兼容——Feelings-OS 裸机环境可直接加载
    ✅ 变长整数编码——u32 强度值通常 < 100，只占 1 字节
    ✅ wire format 简单——Go 端可以手写 encoder（~100 行）
    ✅ 哈希确定性——同一份 FsirDoc 永远产生同一份字节流
    → 刚好够。没有多余。
```

### wire format 速览

```
FsirDoc {
    meta: FsirMeta {
        animi_version: "0.1.0",        // varint(5) + UTF-8 bytes
        compiled_at: "2026-...",        // varint(20) + UTF-8 bytes
        source_hash: Some("abc123..."), // 1 byte tag + varint(64) + UTF-8
        ...
    }
    name: "calm",                       // varint(4) + UTF-8
    mix: FsirMix {
        main: "calm_meditative",        // varint(16) + UTF-8
        accents: [                      // varint(2) = 2 elements
            FsirAccent {
                atom: "belonging",      // varint(9) + UTF-8
                ratio: 0.3,             // 8 bytes LE f64
            },
            ...
        ]
    }
    shape: FsirShape { name: "..." },
    intensity: FsirIntensity { min: 15, max: 45 },  // varint(15) + varint(45)
}
```

无分隔符。无填充。无 Tag。字段顺序 = struct 定义顺序——Postcard 和 Serde 的契约。

### Go 端 encoder 实现要点

Go Server 需要生成相同的字节流。Postcard wire format 的核心规则（< 2 页）：

| Serde 类型 | Postcard 编码 |
|-----------|-------------|
| `bool` | 1 byte: `0x00` = false, `0x01` = true |
| `u32` | unsigned varint (1-5 bytes) |
| `f64` | 8 bytes, little-endian |
| `String` | varint(len) + UTF-8 bytes |
| `Option<T>` | `Some` = `0x01` + T bytes; `None` = `0x00` |
| `Vec<T>` | varint(len) + elements (each T encoded in place) |
| `struct` | fields concatenated in declaration order (no delimiters) |

Go 端需要一个 `PostcardEncoder` struct + `WriteVarint` / `WriteString` / `WriteF64LE` 方法。推荐文件：`feelings-server/internal/fsir/postcard.go`（约 100 行）。

### Rust 端反序列化——`#[no_std]` 就绪

```rust
// Feelings-OS 裸机环境
let fsir: FsirDoc = postcard::from_bytes(&cache_slab)?;
// 零 panic。零 unwrap。Err → oi! 拒绝。
```

Postcard 不依赖 `std`，只需要 `alloc`（`Vec<u8>` / `String`）。Feelings-OS 的 `mempoold` 提供物理连续内存池——`Vec` 的 allocator 映射到 `mempoold` 的 Slab 层即可。

---

## 双格式并存策略

FSIR 同时支持 JSON 和 Postcard。各自的使用场景：

```
JSON
    调试——人类可读，Git diff 可审查
    CLI 输出——animi 默认输出 .json
    创作者审查——感受包的语义验证

Postcard 二进制
    Go Server 预编译 → 设备缓存
    设备启动加载——低延迟，no_std
    双流水线——后台预交织的产物格式
```

JSON 和 Postcard 之间没有转换——它们都是同一份 `FsirDoc` 的序列化。`from_json` 和 `from_binary` 产出的结构体完全等价。

---

## 和已有架构的咬合

```
双流水线（ADR 004）
    后台：Go Server → Postcard encoder → FSIR 二进制缓存
    前台：Feelings-OS → mempoold 加载 → postcard::from_bytes → PBM 左乘

四层 IR（ADR 002）
    FSIR 二进制 = 感受结构的"通用本体"的物理表示
    PSIR / DSIR / ESIR 后续也可按同样的 Postcard 格式序列化

SPL 锚定
    FSIR 二进制的 SHA-256 = 部署在设备上的 FSIR 的不可篡改指纹
    和源码 SHA-256 + Registry SHA-256 三位一体锚定
```

---

## 禁止事项

```
❌ 用 JSON 做设备缓存——二进制是唯一的高性能路径
❌ 用手写字节布局替代 postcard——Serde derive 是唯一源
❌ Go 端用不同的序列化格式——必须和 postcard wire format 位一致
❌ 引入 FlatBuffers schema 编译器——构建复杂度不应为 3 层字段树引入
```

---

*FSIR 是感受结构的通用本体。JSON 是它的"人类可读面"。Postcard 是它的"设备可读面"。同一个东西。两种光。一帧不差。*
