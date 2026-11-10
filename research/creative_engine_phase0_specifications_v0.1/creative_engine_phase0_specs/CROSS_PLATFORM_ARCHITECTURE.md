# 10. Cross-Platform Architecture Specification

**Status:** Draft 0.1

## 10.1 Platform strategy

- Linux is the primary development environment.
- Desktop UI: Iced on Linux, Windows, and macOS.
- Mobile: native Android and iOS host shells, with Rust core exposed through a stable FFI boundary.
- CLI and MCP are headless Rust applications.
- Web is not a 1.0 commitment; keep domain/core free of assumptions that prevent a future web target.

Iced's commonly documented target set is desktop and web; do not assume it supplies production-ready Android/iOS UI. wgpu can support mobile graphics backends, but window/surface lifecycle and native application integration remain host responsibilities. Verify current framework support before implementation.

## 10.2 Platform abstraction

Define traits/services for:
- file open/save and document picker;
- app data/cache directories;
- clipboard;
- secure credential storage;
- stylus and pointer input;
- haptics;
- share sheet;
- notifications where needed;
- display scale and safe areas;
- GPU surface creation and lifecycle.

## 10.3 Mobile FFI

Expose a narrow C ABI or UniFFI-generated interface. Do not expose internal Rust structs directly. Use opaque handles and versioned request/response DTOs. Define thread ownership, callback lifetime, cancellation, and error conversion.

## 10.4 Graphics

- Desktop: wgpu surface integrated with Iced or a carefully isolated custom rendering path.
- Android: validate Vulkan first, GLES fallback as needed.
- iOS: Metal path through wgpu/native integration.
- Renderer must tolerate surface recreation, app suspension, device loss, and orientation changes.

## 10.5 Input

Normalize stylus/touch events in platform adapters. Preserve pressure, tilt, tool type, timestamps, and eraser state when supplied. Hardware capabilities are queried at runtime.

## 10.6 Storage

- Desktop: normal filesystem paths and atomic save.
- Android/iOS: document-provider APIs and app sandbox; do not assume unrestricted filesystem paths.
- Use content URIs/bookmarks/security-scoped access where required.
- Keep temporary/cache data separate from user documents.

## 10.7 Build and release

- CI matrix for Linux, Windows, macOS, Android, and iOS compilation.
- Linux development should be able to run core tests and desktop builds without mobile SDKs.
- Mobile release builds require platform toolchains and signing credentials.
- Reproducible dependency lockfiles and artifact provenance.

## 10.8 Platform parity

Maintain a capability matrix. A feature may be unavailable on a platform only for a documented reason, with a fallback or clear UI explanation. Do not silently pretend unsupported stylus signals exist.

## 10.9 Early risk spikes

Before full implementation, prove:
1. Iced desktop shell + wgpu canvas integration.
2. Android native host + Rust core + GPU surface + stylus input.
3. iOS native host + Rust core + Metal surface.
4. File picker and atomic save behaviour on mobile.
5. FFI call/callback and cancellation behaviour.
