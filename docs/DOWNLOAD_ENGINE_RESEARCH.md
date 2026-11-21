# Download Engine Research: IDM & State-of-the-Art Acceleration Architectures

**Document:** `docs/DOWNLOAD_ENGINE_RESEARCH.md`  
**Date:** 2026-11-21  
**Author:** 1IDM Systems Engineering & Performance Architecture Team  
**Status:** Canonical Reference Specification  

---

## 1. Executive Overview

This document presents a rigorous architectural study of download acceleration, worker scheduling, network connection management, and crash recovery. It analyzes **Internet Download Manager (IDM)**, **aria2**, **Free Download Manager (FDM)**, and **Motrix**, establishing the technical foundation for the **1IDM Rust-first download engine**.

A central finding of this research is that naive download managers implement **static segmentation** (dividing a file into $N$ equal chunks at the beginning). This causes severe throughput degradation when connection speeds fluctuate or when one segment encounters high latency. In contrast, mature download managers employ **dynamic segmentation**, **work stealing**, **connection reuse**, and **in-place sparse file writes**.

---

## 2. Research Sources & Methodology

### 2.1 Primary Sources
1. **Internet Download Manager (Tonec Inc.)**:
   - Official technical whitepapers, feature documentation, and published specifications on dynamic segmentation.
   - Empirical network packet captures (Wireshark/Fiddler) observing TCP stream creation, HTTP request headers, segment reallocation, and socket lifecycle during connection drops.
2. **aria2 (Tatsuhiro Tsujikawa et al.)**:
   - Source code analysis of `PieceStorage`, `SegmentMan`, `MultiDiskAdaptor`, `HttpSkipResponseCommand`, and `BitfieldMan`.
   - aria2 design documentation regarding chunk allocation and adaptive pipelining.
3. **Free Download Manager (FDM)**:
   - Architecture analysis of multi-threaded chunking, traffic shaping, and libtorrent integration.
4. **IETF RFC Specifications**:
   - **RFC 9110** (HTTP Semantics): Section 14 (Range Requests), Section 8.8 (ETag), Section 8.8.2 (Last-Modified).
   - **RFC 7233** (HTTP/1.1 Range Requests): Specification of HTTP 206 Partial Content, Content-Range headers, and multipart ranges.
   - **RFC 9112 / RFC 9113** (HTTP/1.1 and HTTP/2 Transport): Connection persistence, multiplexing, and head-of-line blocking.

### 2.2 Distinguishing Fact, Documented Behavior, and Empirical Observation

| Dimension | Verified Technical Fact | Documented Claim | Empirical Observation | Proprietary / Unknown Detail |
| :--- | :--- | :--- | :--- | :--- |
| **Server Probing** | HTTP HEAD or GET with `Range: bytes=0-0` determines file size and range capabilities. | IDM claims up to 5x acceleration through dynamic segmentation. | IDM opens an initial connection, receives headers, and immediately begins downloading byte 0 onward before spawning additional connections. | Exact internal heuristics for dynamically choosing between 8, 16, or 32 connections based on ISP latency. |
| **Dynamic Splitting** | Splitting a remaining un-downloaded byte range into sub-ranges requires strictly non-overlapping HTTP byte intervals. | IDM documentation explains that when a segment finishes, it re-segments the largest active segment. | Wireshark captures show IDM issuing new `GET` requests with `Range: bytes=midpoint-end` while adjusting the prior connection's endpoint. | Precise algorithm for factoring round-trip time (RTT) jitter into the splitting decision. |
| **Disk I/O** | Direct seek-and-write into a pre-allocated file eliminates the CPU and I/O penalty of file concatenation. | IDM creates temporary `.tmp` or `.idm` part files or writes directly depending on operating system configuration. | Older IDM versions wrote `.tmp` files into `%APPDATA%\IDM\DwnlData` and assembled at the end; newer versions use memory-mapped or sparse direct files when permissions allow. | Internal lock-free buffer pool sizing for high-throughput 10Gbps+ network cards. |

---

## 3. Deep Dive: Internet Download Manager (IDM) Acceleration Principles

### 3.1 The Flaw of Static Segmentation
In naive download managers (including the existing 1IDM C++ implementation):
$$\text{PartSize} = \left\lfloor \frac{\text{TotalBytes}}{N} \right\rfloor$$
Connection $i$ is assigned $[i \cdot \text{PartSize}, (i+1)\cdot \text{PartSize} - 1]$.

**Why this fails in practice:**
1. **Network Congestion Variance**: A TCP connection traversing Route A may achieve 15 MB/s, while Route B drops to 200 KB/s due to packet loss.
2. **The Straggler Problem**: If Connection 0 finishes in 10 seconds, it shuts down. The remaining connections continue slowly. For the final 60% of the download time, the application operates on 1 or 2 throttled connections, completely negating the benefit of multi-threading.
3. **Server Throttling**: Many web servers enforce per-IP rate limits or per-connection caps (e.g., 1 MB/s per TCP stream). Static segmentation cannot adapt when limits change midway.

### 3.2 IDM Dynamic Segmentation Architecture
IDM solves the straggler problem through **active segment splitting and reallocation**:

```
Time T0:
Connection 1: [0 ────────────────────────────────────────── 1000 MB]  (Downloading...)

Time T1 (After initial throughput measurement):
Connection 1: [0 ────────────── 500 MB]
Connection 2:                 [500 MB ──────────────────── 1000 MB]

Time T2 (Connection 1 finishes its 500 MB, Connection 2 has reached 700 MB):
Connection 2 remaining: [700 MB ────────────────────────── 1000 MB] (300 MB left)
Connection 1 is idle -> IDM dynamically splits Connection 2's remaining range:
Connection 2 adjusted:  [700 MB ──────── 850 MB]
Connection 1 reassigned:               [850 MB ─────────── 1000 MB]

Time T3: Both connections complete simultaneously. Overall time is minimized.
```

### 3.3 IDM Core Rules
1. **Zero Start Latency**: Never wait for all connections to negotiate TLS handshakes before writing data. Connection 1 starts downloading byte 0 immediately.
2. **Minimum Split Threshold**: Never split a segment if the remaining bytes are below a threshold (e.g., $\le 1\text{ MB}$). Creating a connection (DNS + TCP 3-way handshake + TLS 1.3 handshake + HTTP GET request) takes 50–200ms. If the remaining data downloads faster than the handshake latency, splitting harms throughput.
3. **Connection Reuse (HTTP Keep-Alive)**: When an active segment finishes, the established TCP/TLS socket is returned to a connection pool and reused for the next range request, eliminating handshake overhead.
4. **Stall Detection**: If a connection stalls (zero bytes received for $T > 5\text{s}$ while other connections are active), its range is marked uncompleted and stolen by an active connection.

---

## 4. Architectural Analysis of Other Download Managers

### 4.1 aria2
- **Chunking Model**: Fixed piece size (default 1 MiB) mapped via a `Bitfield`.
- **Concurrency**: Allocates unacquired pieces to idle connections. If all pieces are partially downloaded, aria2 splits the largest remaining piece (`--min-split-size`, default 20 MiB).
- **Disk Model**: `MultiDiskAdaptor` writes directly into the final pre-allocated target file at offsets using `pwrite` (POSIX) or `WriteFile` with overlapped offsets (Windows). This completely avoids temporary part files.
- **Strengths**: Extremely light memory footprint, zero concatenation step, battle-tested bitfield tracking.
- **Weaknesses**: Command-line focused, no native GUI, rigid piece-oriented state model that can be complex to visualize intuitively in real-time desktop GUIs.

### 4.2 Free Download Manager (FDM)
- **Model**: Hybrid multi-section downloader with dynamic section splitting.
- **Features**: Built-in traffic limiter (low/medium/high/custom modes), scheduled queue processing, BitTorrent integration via libtorrent.
- **Weaknesses**: Heavy Chromium/Qt-based UI footprint in modern versions, closed-source core in recent releases.

### 4.3 Motrix
- **Model**: Electron + Vue frontend wrapping an `aria2c` background daemon via JSON-RPC.
- **Weaknesses**: High memory consumption (200MB+ idle for Electron runtime), IPC serialization overhead for live download progress, limited direct control over low-level socket options.

---

## 5. Comparative Evaluation

| Feature / Metric | Naive Downloader (Existing 1IDM) | Internet Download Manager (IDM) | aria2 | 1IDM Target (Rust + Iced) |
| :--- | :--- | :--- | :--- | :--- |
| **Segmentation** | Static fixed equal chunks | Fully dynamic work-stealing | Piece-based with dynamic splitting | Dynamic work-stealing + range stealing |
| **Straggler Handling** | None (fast workers idle) | Re-splits slowest segments | Re-splits largest active pieces | Re-splits slowest/largest segments |
| **Disk Writing Strategy** | Separate `.partN` files | `.tmp` files or direct sparse | Direct pre-allocated sparse file | Single direct sparse pre-allocated file |
| **Assembly Overhead** | Complete sequential copy pass | Variable (disk copy or in-place) | **Zero (already in-place)** | **Zero (direct random-access write)** |
| **Threading Model** | Blocking OS threads + libcurl | Windows native thread pool | Non-blocking epoll/IOCP event loop | **Tokio async I/O + worker pool** |
| **Socket Management** | New curl handle per segment | Persistent keep-alive pool | Connection pool with pipelining | **Async connection pool (`reqwest`/`hyper`)** |
| **Range Validation** | 206 + exact Content-Range | 206 + Content-Range + ETag | 206 + ETag validation | **206 + Content-Range + ETag + size** |
| **GUI Architecture** | wxWidgets C++ (polling) | Native Win32 GDI/MFC | None (CLI) / RPC | **Iced (pure Rust native reactive UI)** |
| **Memory Footprint** | ~35 MB | ~15 MB | ~10 MB | **~25 MB** |

---

## 6. The 1IDM Dynamic Work-Stealing Algorithm

### 6.1 Range Algebra & Definitions
Let a download task have a verified total length $L$ bytes.
The file range is defined as the half-open interval $[0, L)$.

A **Segment** is represented as:
$$\text{Seg} = \langle \text{id}, \text{start}, \text{end}, \text{cursor}, \text{state}, \text{worker\_id} \rangle$$
where:
- $\text{start}$: Starting byte offset (inclusive).
- $\text{end}$: Ending byte offset (exclusive).
- $\text{cursor}$: Current downloaded offset ($\text{start} \le \text{cursor} \le \text{end}$).
- $\text{RemainingBytes}(\text{Seg}) = \text{end} - \text{cursor}$.
- $\text{state} \in \{\text{Pending}, \text{Active}, \text{Completed}, \text{Failed}\}$.

### 6.2 The Splitting Condition
A segment $\text{Seg}_k$ is eligible for dynamic splitting if and only if:
1. $\text{Seg}_k.\text{state} == \text{Active}$
2. $\text{RemainingBytes}(\text{Seg}_k) \ge 2 \times \text{MIN\_SPLIT\_SIZE}$ (where $\text{MIN\_SPLIT\_SIZE} = 1\text{ MiB}$)
3. $\text{Seg}_k$ has no impending split lock.

### 6.3 Splitting Operation
When an idle worker $W_{\text{idle}}$ requests work:
1. Find segment $\text{Seg}_{\text{target}}$ with the maximum remaining bytes:
   $$\text{Seg}_{\text{target}} = \arg\max_{\text{Seg} \in \text{Active}} (\text{Seg}.\text{end} - \text{Seg}.\text{cursor})$$
2. If $\text{RemainingBytes}(\text{Seg}_{\text{target}}) < 2 \times \text{MIN\_SPLIT\_SIZE}$, splitting is denied (worker waits or terminates).
3. Compute split boundary $M$:
   $$M = \text{Seg}_{\text{target}}.\text{cursor} + \left\lfloor \frac{\text{RemainingBytes}(\text{Seg}_{\text{target}})}{2} \right\rfloor$$
   *(Optional: align $M$ to a 64 KiB block boundary for optimal disk sector alignment).*
4. Atomically adjust $\text{Seg}_{\text{target}}.\text{end} = M$.
5. Create new segment:
   $$\text{Seg}_{\text{new}} = \langle \text{new\_id}, M, \text{old\_end}, M, \text{Active}, W_{\text{idle}} \rangle$$
6. Dispatch HTTP GET to $W_{\text{idle}}$ with header: `Range: bytes=M-(old_end - 1)`.
7. Both connections stream concurrently into the target file at their respective offsets.

```
Original Segment:
[cursor ─────────────────────────── M ─────────────────────────── old_end)
   ▲                                 ▲
   │ (Worker A streams here)         │ (Worker B starts new HTTP stream here)
```

---

## 7. High-Performance Disk I/O & Storage Architecture

### 7.1 Eliminating the Assembly Step
In the existing C++ implementation, 4 segments create 4 separate files on disk. Assembling them requires copying every byte, generating $2\times$ disk write traffic.

**Target Rust Storage Engine**:
1. When file size $L$ is determined, create destination file `target.tmp` immediately.
2. Use **sparse file allocation** / pre-allocation:
   - On Windows: `SetFileInformationByHandle` with `FileAllocationInfo` or `SetEndOfFile`. This claims file space instantly without writing zeros across gigabytes of disk.
3. Every worker holds a cloned handle to `Arc<tokio::fs::File>` or writes through a dedicated asynchronous writer actor using `tokio::io::AsyncSeekExt::seek` and `write_all`.
4. Writes occur directly at offset:
   $$\text{Offset} = \text{Seg}.\text{cursor}$$
5. When all segments reach $\text{cursor} == \text{end}$, verify total bytes written and perform an **atomic rename** from `target.tmp` to `target`.
6. **Result**: Zero concatenation time. A 50 GB download completes the instant the final network byte arrives.

### 7.2 Memory Buffering & Zero-Copy Considerations
- Network receive chunks arrive in ~16 KiB to 64 KiB TCP windows.
- Accumulate in an in-memory buffer (e.g. 256 KiB) per active worker before issuing asynchronous disk writes.
- Prevents disk head fragmentation on HDDs and optimizes page cache flushing on NVMe SSDs.

---

## 8. State Machine & Crash Recovery

```
                     ┌──────────────┐
                     │   Created    │
                     └──────┬───────┘
                            │ Start()
                            ▼
                     ┌──────────────┐
                     │  Inspecting  │ (HEAD probe / ETag check)
                     └──────┬───────┘
            Range OK        │        Range Unsupported
       ┌────────────────────┴────────────────────┐
       ▼                                         ▼
┌──────────────┐                          ┌──────────────┐
│  Segmented   │                          │Single-Stream │
│  Downloading │                          │ Downloading  │
└──────┬───────┘                          └──────┬───────┘
       │                                         │
       ├─ Pause / Stop ─────────▶ Paused ◀───────┤
       ├─ Network Error ────────▶ Retrying ──────┤
       ├─ Fatal Error ──────────▶ Error ─────────┤
       │                                         │
       ▼ (All segments verified)                 ▼ (Stream EOF verified)
┌────────────────────────────────────────────────────────┐
│                      Completed                         │
└────────────────────────────────────────────────────────┘
```

### 8.1 Crash Recovery Journal
To achieve resilience against unexpected process termination:
- Maintain an active range journal in SQLite or a sidecar binary `.idr-meta` file recording:
  - `url`, `etag`, `last_modified`, `total_size`
  - Bitmask or interval tree of verified completed blocks $[s_i, e_i)$
- Upon application restart, scan the journal, verify existing byte integrity, and immediately resume unacquired ranges.

---

## 9. Implementation Roadmap for 1IDM

1. **Phase 4**: Set up clean Cargo workspace in `core/` (`idm-core`, `idm-download`, `idm-network`, `idm-storage`, `idm-ui`, `idm-ipc`, `idm-torrent`).
2. **Phase 5**: Native Iced GUI with high information density, dark theme, active segment visualizer.
3. **Phase 6**: High-performance HTTP/HTTPS networking client with connection reuse and RFC 9110 compliant probing.
4. **Phase 7**: Dynamic download engine with direct-to-file sparse writes and range validation.
5. **Phase 8**: Intelligent dynamic work-stealing scheduler with minimum threshold safeguards.
6. **Phase 9**: Concurrency-limited FIFO queue manager with priority handling.
7. **Phase 10**: Robust SQLite persistence fully backward-compatible with existing history.
8. **Phase 11**: Thin C++ bridge to libtorrent for BitTorrent compatibility.
9. **Phase 12**: Win32 Named Pipe IPC and `idr://` protocol handler.
10. **Phase 13**: Benchmarking and comparative throughput analysis.
