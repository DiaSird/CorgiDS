# Nintendo DS エミュレータ - 実装ガイド

各フェーズの設計に基づいて、具体的なソースコード実装を解説します。
なぜその実装をするのか、どのような設計思想があるのかを詳しく説明します。

---

## Phase 0: FreeBIOS Development (完了 ✅)

### 目的

独自のフリー BIOS 実装により、ユーザーが物理デバイスから BIOS をダンプせずにエミュレータを使用可能にする。

### 実装ファイル

**freebios.hpp / freebios.cpp**

```cpp
// freebios.hpp
#ifndef FREEBIOS_HPP
#define FREEBIOS_HPP

#include <cstdint>
#include <vector>

class FreeBIOS {
private:
    // BIOS全体は256バイト以下の簡潔な実装
    std::vector<uint8_t> arm9_bios;
    std::vector<uint8_t> arm7_bios;

public:
    FreeBIOS();

    // BIOS生成
    void generate_arm9_bios();
    void generate_arm7_bios();

    // BIOSデータ取得
    const uint8_t* get_arm9_bios() const;
    const uint8_t* get_arm7_bios() const;

    uint32_t get_arm9_bios_size() const;
    uint32_t get_arm7_bios_size() const;
};

#endif
```

### なぜこの実装か？

1. **最小限の実装で十分**

   - DS BIOS の本質的な役割: システム初期化と割り込みハンドラ
   - 完全エミュレーションは不要
   - 初期化シーケンスを最小限実装するだけで十分

2. **メモリ効率**

   - BIOS 全体は数 KB 程度
   - vector で動的管理により、ポインタ操作を安全に

3. **拡張性**
   - 後で実装が必要になれば追加可能
   - 構造化されているため修正容易

---

## Phase 1: Foundation Setup - Memory & Constants

### 目的

メモリマップの定義と定数管理により、物理メモリレイアウトをシステム全体で一貫性を持たせる。

### 実装ファイル

#### constants.hpp

```cpp
// constants.hpp
#ifndef CONSTANTS_HPP
#define CONSTANTS_HPP

#include <cstdint>

// ============================================
// メモリアドレス定義
// ============================================

// ARM9 メモリマップ
namespace ARM9 {
    // BIOS領域 (4KB)
    const uint32_t BIOS_START = 0x00000000;
    const uint32_t BIOS_SIZE  = 0x1000;

    // メインメモリ (4MB)
    const uint32_t MAIN_RAM_START = 0x02000000;
    const uint32_t MAIN_RAM_SIZE  = 0x400000;

    // 内部WRAM (32KB)
    const uint32_t WRAM_START = 0x03000000;
    const uint32_t WRAM_SIZE  = 0x8000;

    // I/O レジスタ (64KB)
    const uint32_t IO_START = 0x04000000;
    const uint32_t IO_SIZE  = 0x10000;

    // パレットメモリ (1KB)
    const uint32_t PALETTE_START = 0x05000000;
    const uint32_t PALETTE_SIZE  = 0x400;

    // VRAM (8MB)
    const uint32_t VRAM_START = 0x06000000;
    const uint32_t VRAM_SIZE  = 0x800000;

    // OAMメモリ (2KB)
    const uint32_t OAM_START = 0x07000000;
    const uint32_t OAM_SIZE  = 0x800;

    // ROM領域
    const uint32_t ROM_START = 0x08000000;
}

// ARM7 メモリマップ
namespace ARM7 {
    // BIOS領域 (16KB)
    const uint32_t BIOS_START = 0x00000000;
    const uint32_t BIOS_SIZE  = 0x4000;

    // メインメモリ (一部)
    const uint32_t MAIN_RAM_START = 0x02000000;
    const uint32_t MAIN_RAM_SIZE  = 0x400000;

    // ARM7専用WRAM (64KB)
    const uint32_t WRAM_START = 0x03800000;
    const uint32_t WRAM_SIZE  = 0x10000;

    // 共有WRAM (32KB)
    const uint32_t SHARED_WRAM_START = 0x02800000;
    const uint32_t SHARED_WRAM_SIZE  = 0x8000;
}

// ============================================
// レジスタオフセット (I/Oレジスタ)
// ============================================

// Display コントロール
const uint32_t DISPCNT = 0x04000000;
const uint32_t DISPSTAT = 0x04000004;
const uint32_t VCOUNT = 0x04000006;

// キー入力
const uint32_t KEYINPUT = 0x04000130;

// 割り込み管理
const uint32_t IE = 0x04000200;      // 割り込み有効
const uint32_t IF = 0x04000202;      // 割り込みフラグ
const uint32_t IME = 0x04000208;     // マスター割り込み有効

// ============================================
// CPU関連定数
// ============================================

enum class PSR_MODE : uint8_t {
    USER       = 0x10,
    FIQ        = 0x11,
    IRQ        = 0x12,
    SUPERVISOR = 0x13,
    ABORT      = 0x17,
    UNDEFINED  = 0x1B,
    SYSTEM     = 0x1F
};

// フラグビット位置 (CPSR)
const uint32_t FLAG_N = 0x80000000;  // ネガティブ
const uint32_t FLAG_Z = 0x40000000;  // ゼロ
const uint32_t FLAG_C = 0x20000000;  // キャリー
const uint32_t FLAG_V = 0x10000000;  // オーバーフロー

// CPU周波数
const uint32_t CPU_CLOCK_HZ = 67737600;  // 67.7 MHz

// ============================================
// GPU関連定数
// ============================================

// スクリーン解像度
const int SCREEN_WIDTH = 256;
const int SCREEN_HEIGHT = 192;
const int SCREEN_SIZE = SCREEN_WIDTH * SCREEN_HEIGHT;

// フレームレート
const int FPS = 60;
const int CYCLES_PER_FRAME = CPU_CLOCK_HZ / FPS;

#endif
```

#### memory.hpp

```cpp
// memory.hpp
#ifndef MEMORY_HPP
#define MEMORY_HPP

#include <cstdint>
#include <array>
#include "constants.hpp"

class Memory {
private:
    // メモリ領域 (すべてのARMプロセッサが共有)
    std::array<uint8_t, ARM9::BIOS_SIZE> arm9_bios;
    std::array<uint8_t, ARM7::BIOS_SIZE> arm7_bios;

    std::array<uint8_t, ARM9::MAIN_RAM_SIZE> main_ram;
    std::array<uint8_t, ARM7::WRAM_SIZE> arm7_wram;
    std::array<uint8_t, ARM9::VRAM_SIZE> vram;
    std::array<uint8_t, ARM9::OAM_SIZE> oam;
    std::array<uint8_t, ARM9::PALETTE_SIZE> palette;

    std::array<uint8_t, ARM9::IO_SIZE> io_regs;

    // ROM メモリ (別途ロード)
    std::vector<uint8_t> rom_data;

public:
    Memory();

    // ============================================
    // メモリ読み取り (8/16/32ビット)
    // ============================================

    // 8ビット読み取り
    uint8_t read8(uint32_t addr);

    // 16ビット読み取り (リトルエンディアン)
    uint16_t read16(uint32_t addr);

    // 32ビット読み取り (リトルエンディアン)
    uint32_t read32(uint32_t addr);

    // ============================================
    // メモリ書き込み (8/16/32ビット)
    // ============================================

    void write8(uint32_t addr, uint8_t value);
    void write16(uint32_t addr, uint16_t value);
    void write32(uint32_t addr, uint32_t value);

    // ============================================
    // ROM ロード
    // ============================================

    bool load_rom(const std::string& path);

    // ============================================
    // メモリダンプ (デバッグ用)
    // ============================================

    void dump_memory(uint32_t start, uint32_t size, const char* filename);
};

#endif
```

#### memory.cpp

```cpp
// memory.cpp
#include "memory.hpp"
#include <iostream>
#include <fstream>

Memory::Memory() {
    // メモリ初期化 (0で埋める)
    arm9_bios.fill(0);
    arm7_bios.fill(0);
    main_ram.fill(0);
    vram.fill(0);
    // ... 他も同じ
}

uint8_t Memory::read8(uint32_t addr) {
    // アドレス範囲をチェックしてメモリをマップ
    if (addr >= ARM9::BIOS_START && addr < ARM9::BIOS_START + ARM9::BIOS_SIZE) {
        return arm9_bios[addr - ARM9::BIOS_START];
    }
    else if (addr >= ARM9::MAIN_RAM_START && addr < ARM9::MAIN_RAM_START + ARM9::MAIN_RAM_SIZE) {
        return main_ram[addr - ARM9::MAIN_RAM_START];
    }
    else if (addr >= ARM9::VRAM_START && addr < ARM9::VRAM_START + ARM9::VRAM_SIZE) {
        return vram[addr - ARM9::VRAM_START];
    }
    else if (addr >= ARM9::ROM_START && addr < ARM9::ROM_START + rom_data.size()) {
        return rom_data[addr - ARM9::ROM_START];
    }
    else {
        std::cerr << "Memory::read8 - Invalid address: 0x" << std::hex << addr << std::endl;
        return 0;
    }
}

uint16_t Memory::read16(uint32_t addr) {
    // リトルエンディアンで2バイト読み取り
    uint8_t lo = read8(addr);
    uint8_t hi = read8(addr + 1);
    return (hi << 8) | lo;
}

uint32_t Memory::read32(uint32_t addr) {
    // リトルエンディアンで4バイト読み取り
    uint16_t lo = read16(addr);
    uint16_t hi = read16(addr + 2);
    return (hi << 16) | lo;
}

void Memory::write8(uint32_t addr, uint8_t value) {
    if (addr >= ARM9::MAIN_RAM_START && addr < ARM9::MAIN_RAM_START + ARM9::MAIN_RAM_SIZE) {
        main_ram[addr - ARM9::MAIN_RAM_START] = value;
    }
    else if (addr >= ARM9::VRAM_START && addr < ARM9::VRAM_START + ARM9::VRAM_SIZE) {
        vram[addr - ARM9::VRAM_START] = value;
    }
    // ... 他も同様
}

uint16_t Memory::write16(uint32_t addr, uint16_t value) {
    write8(addr, value & 0xFF);
    write8(addr + 1, (value >> 8) & 0xFF);
}

uint32_t Memory::write32(uint32_t addr, uint32_t value) {
    write16(addr, value & 0xFFFF);
    write16(addr + 2, (value >> 16) & 0xFFFF);
}

bool Memory::load_rom(const std::string& path) {
    std::ifstream file(path, std::ios::binary | std::ios::ate);
    if (!file.is_open()) {
        std::cerr << "Failed to load ROM: " << path << std::endl;
        return false;
    }

    std::streamsize size = file.tellg();
    file.seekg(0, std::ios::beg);

    rom_data.resize(size);
    if (!file.read(reinterpret_cast<char*>(rom_data.data()), size)) {
        std::cerr << "Failed to read ROM file" << std::endl;
        return false;
    }

    return true;
}
```

### なぜこの実装か？

1. **定数の一元管理**

   - メモリアドレスや定数が複数箇所で散らばると、管理困難・バグ源
   - constants.hpp で一元管理することで、修正時に 1 箇所のみ変更

2. **std::array の利用**

   - C 配列より安全（ボウンダリチェック可能）
   - std::vector より効率的（サイズが固定だから）

3. **メモリマッピングの仕組み**

   - CPU が読み書きするアドレスと物理メモリをマッピング
   - 複数の"ウィンドウ"（ビュー）により同じメモリに複数からアクセス可能

4. **リトルエンディアン対応**
   - ARM はリトルエンディアンが標準
   - 16/32 ビット読み書きで適切に変換

---

## Phase 2: CPU Core Implementation

### 目的

命令実行エンジンの基本骨組みを構築し、シンプルな命令から複雑な命令へと段階的に追加できる基盤を作る。

### 実装ファイル

#### cpu.hpp

```cpp
// cpu.hpp
#ifndef CPU_HPP
#define CPU_HPP

#include <cstdint>
#include <string>
#include "constants.hpp"

class Memory;
class Emulator;

// CPUの実行モード
enum class ExecutionMode {
    ARM,    // 32ビットARM命令
    THUMB   // 16ビットThumb命令
};

class ARM_CPU {
private:
    // 汎用レジスタ (R0-R15)
    // R15 = PC (プログラムカウンタ)
    uint32_t r[16];

    // 現在のプログラムステータスレジスタ
    struct {
        // フラグ
        bool N;  // ネガティブフラグ
        bool Z;  // ゼロフラグ
        bool C;  // キャリーフラグ
        bool V;  // オーバーフローフラグ

        // モード制御
        bool I;  // IRQ割り込み無効
        bool F;  // FIQ割り込み無効

        // 実行モード
        ExecutionMode mode;
        PSR_MODE privilege_mode;
    } cpsr;

    // SPSR (保存されたプログラムステータスレジスタ)
    // 例外発生時にCPSRを保存
    uint32_t spsr[6];

    // CPU識別子
    int cpu_id;  // 0 = ARM9, 1 = ARM7

    // 実行時の情報
    uint32_t current_instruction;
    bool halted;
    uint64_t cycle_count;

    // メモリ・エミュレータへのポインタ
    Memory* memory;
    Emulator* emulator;

public:
    ARM_CPU(Emulator* emu, int id);

    // ============================================
    // CPUライフサイクル
    // ============================================

    void power_on();
    void execute_cycle();
    void halt();
    void wake();

    // ============================================
    // レジスタ操作
    // ============================================

    uint32_t get_register(int idx) const;
    void set_register(int idx, uint32_t value);

    uint32_t get_pc() const { return r[15]; }
    void set_pc(uint32_t value) { r[15] = value; }

    // フラグ操作
    void set_flag_z(bool value) { cpsr.Z = value; }
    void set_flag_n(bool value) { cpsr.N = value; }
    void set_flag_c(bool value) { cpsr.C = value; }
    void set_flag_v(bool value) { cpsr.V = value; }

    bool get_flag_z() const { return cpsr.Z; }
    bool get_flag_n() const { return cpsr.N; }
    bool get_flag_c() const { return cpsr.C; }
    bool get_flag_v() const { return cpsr.V; }

    // ============================================
    // 命令実行 (次フェーズで拡張)
    // ============================================

    void execute_arm_instruction(uint32_t instr);
    void execute_thumb_instruction(uint16_t instr);

    // ============================================
    // 条件判定
    // ============================================

    bool check_condition(uint8_t cond);

    // ============================================
    // 割り込み処理 (Phase 8で詳細化)
    // ============================================

    void handle_interrupt(int irq_type);

    // ============================================
    // デバッグ
    // ============================================

    void print_registers();
    std::string disassemble(uint32_t addr);
};

#endif
```

#### cpu.cpp (基本実装)

```cpp
// cpu.cpp
#include "cpu.hpp"
#include "memory.hpp"
#include "emulator.hpp"
#include <iostream>
#include <iomanip>

ARM_CPU::ARM_CPU(Emulator* emu, int id)
    : cpu_id(id), emulator(emu), halted(false), cycle_count(0) {
    // レジスタ初期化
    for (int i = 0; i < 16; i++) {
        r[i] = 0;
    }

    // CPSR初期化
    cpsr.N = cpsr.Z = cpsr.C = cpsr.V = false;
    cpsr.I = cpsr.F = false;
    cpsr.mode = ExecutionMode::ARM;
    cpsr.privilege_mode = PSR_MODE::SUPERVISOR;
}

void ARM_CPU::power_on() {
    // CPU起動時の初期化
    // プログラムカウンタをBIOSの開始位置に設定
    r[15] = 0x00000000;  // BIOS start
    halted = false;
}

void ARM_CPU::execute_cycle() {
    if (halted) {
        return;
    }

    // メモリからPC位置の命令をフェッチ
    if (cpsr.mode == ExecutionMode::ARM) {
        // 32ビット命令フェッチ
        current_instruction = memory->read32(r[15]);

        // 命令を実行
        execute_arm_instruction(current_instruction);

        // PC を次の命令へ (ARM は 4バイト)
        r[15] += 4;
    } else {
        // 16ビット命令フェッチ
        uint16_t instr = memory->read16(r[15]);

        // 命令を実行
        execute_thumb_instruction(instr);

        // PC を次の命令へ (Thumb は 2バイト)
        r[15] += 2;
    }

    cycle_count++;
}

uint32_t ARM_CPU::get_register(int idx) const {
    if (idx < 0 || idx > 15) {
        std::cerr << "Invalid register index: " << idx << std::endl;
        return 0;
    }
    return r[idx];
}

void ARM_CPU::set_register(int idx, uint32_t value) {
    if (idx < 0 || idx > 15) {
        std::cerr << "Invalid register index: " << idx << std::endl;
        return;
    }
    r[idx] = value;
}

bool ARM_CPU::check_condition(uint8_t cond) {
    // ARM命令の上位4ビットで条件フラグをチェック
    switch (cond) {
        case 0x0: return cpsr.Z;           // EQ: Equal
        case 0x1: return !cpsr.Z;          // NE: Not equal
        case 0x2: return cpsr.C;           // CS: Carry set
        case 0x3: return !cpsr.C;          // CC: Carry clear
        case 0x4: return cpsr.N;           // MI: Minus
        case 0x5: return !cpsr.N;          // PL: Plus
        case 0x6: return cpsr.V;           // VS: Overflow
        case 0x7: return !cpsr.V;          // VC: No overflow
        case 0x8: return cpsr.C && !cpsr.Z; // HI: Higher
        case 0x9: return !cpsr.C || cpsr.Z; // LS: Lower or same
        case 0xA: return cpsr.N == cpsr.V; // GE: Greater or equal
        case 0xB: return cpsr.N != cpsr.V; // LT: Less than
        case 0xC: return !cpsr.Z && (cpsr.N == cpsr.V); // GT: Greater than
        case 0xD: return cpsr.Z || (cpsr.N != cpsr.V); // LE: Less or equal
        case 0xE: return true;             // AL: Always
        case 0xF: return false;            // Never (reserved)
        default: return true;
    }
}

void ARM_CPU::print_registers() {
    std::cout << "=== CPU Registers ===" << std::endl;
    for (int i = 0; i < 16; i++) {
        std::cout << "R" << i << " : 0x" << std::hex << std::setw(8)
                  << std::setfill('0') << r[i] << std::endl;
    }
    std::cout << "Flags: N=" << cpsr.N << " Z=" << cpsr.Z
              << " C=" << cpsr.C << " V=" << cpsr.V << std::endl;
}
```

### なぜこの実装か？

1. **条件フラグの明示的な管理**

   - CPSR を構造体で管理することで、各フラグを明確に
   - ビット操作より読みやすく、バグが少ない

2. **段階的な命令追加に対応**

   - execute_arm_instruction() / execute_thumb_instruction() は次フェーズで実装
   - 基本構造は完成，詳細は後で追加可能

3. **cycle_count による同期**

   - フレームごとのサイクル数を管理
   - GPU/タイマーと同期させるため必須

4. **CPU 識別子（cpu_id）**
   - ARM9/ARM7 を同一クラスで管理
   - メモリアドレスマッピングを変える（次フェーズで使用）

---

## Phase 3: Memory / I/O Management

### 目的

I/O レジスタの読み書き処理を統一的に管理し、ゲームが外部機器とやりとりできるようにする。

### 実装ファイル

#### io_registers.hpp

```cpp
// io_registers.hpp
#ifndef IO_REGISTERS_HPP
#define IO_REGISTERS_HPP

#include <cstdint>
#include <functional>
#include <map>

// I/Oレジスタ読み書きハンドラの型
using IOReadHandler = std::function<uint8_t(uint32_t)>;
using IOWriteHandler = std::function<void(uint32_t, uint8_t)>;

class IORegisters {
private:
    // I/Oレジスタストレージ (64KB)
    uint8_t regs[0x10000];

    // レジスタごとの読み書きハンドラ
    std::map<uint32_t, IOReadHandler> read_handlers;
    std::map<uint32_t, IOWriteHandler> write_handlers;

public:
    IORegisters();

    // ============================================
    // レジスタ読み取り
    // ============================================

    uint8_t read8(uint32_t addr);
    uint16_t read16(uint32_t addr);
    uint32_t read32(uint32_t addr);

    // ============================================
    // レジスタ書き込み
    // ============================================

    void write8(uint32_t addr, uint8_t value);
    void write16(uint32_t addr, uint16_t value);
    void write32(uint32_t addr, uint32_t value);

    // ============================================
    // ハンドラ登録
    // ============================================

    void register_read_handler(uint32_t addr, IOReadHandler handler);
    void register_write_handler(uint32_t addr, IOWriteHandler handler);
};

#endif
```

#### io_registers.cpp

```cpp
// io_registers.cpp
#include "io_registers.hpp"
#include <iostream>

IORegisters::IORegisters() {
    // I/Oレジスタを0で初期化
    std::fill(regs, regs + 0x10000, 0);
}

uint8_t IORegisters::read8(uint32_t addr) {
    uint32_t offset = addr - 0x04000000;  // I/O開始アドレス

    // ハンドラが登録されている場合はそれを使用
    auto it = read_handlers.find(offset);
    if (it != read_handlers.end()) {
        return it->second(offset);
    }

    // デフォルト: ストレージから読み取り
    return regs[offset];
}

void IORegisters::write8(uint32_t addr, uint8_t value) {
    uint32_t offset = addr - 0x04000000;

    // ハンドラが登録されている場合はそれを使用
    auto it = write_handlers.find(offset);
    if (it != write_handlers.end()) {
        it->second(offset, value);
        return;
    }

    // デフォルト: ストレージに書き込み
    regs[offset] = value;
}

// 16/32ビット操作も同様に実装...
```

#### display.hpp

```cpp
// display.hpp - ディスプレイ制御
#ifndef DISPLAY_HPP
#define DISPLAY_HPP

#include <cstdint>
#include "io_registers.hpp"

class Display {
private:
    // DISPCNTレジスタ (0x04000000)
    struct {
        uint8_t bg_mode;      // BG描画モード (0-6)
        bool obj_mapping;     // OBJ文字化けマッピング
        bool obj_dim;         // OBJ 次元
        bool page_mode;       // ページモード
        bool forced_blank;    // 画面強制消去
    } dispcnt;

    // VCOUNTレジスタ (0x04000006)
    // 現在のスキャンラインカウンタ (0-227)
    uint8_t vcount;

    // フレームバッファ (メイン)
    uint32_t frame_buffer[256 * 192];

public:
    Display();

    // ============================================
    // レジスタハンドラ
    // ============================================

    uint8_t handle_dispcnt_read(uint32_t offset);
    void handle_dispcnt_write(uint32_t offset, uint8_t value);

    uint8_t handle_vcount_read(uint32_t offset);

    // ============================================
    // フレーム更新
    // ============================================

    void update_scanline(int line);
    uint32_t* get_frame_buffer();
};

#endif
```

### なぜこの実装か？

1. **ハンドラベースの設計**

   - 単純な読み書きだけでなく、副作用のある操作に対応
   - 例: VCOUNT レジスタ読み取り時に割り込み判定など

2. **I/O レジスタの仮想化**

   - 物理レジスタと論理的な意味のあるフィールドを分離
   - コード可読性向上，バグ減少

3. **Display クラスの分離**
   - GPU 処理を独立したモジュールにすることで責任を明確化
   - 複雑な GPU 処理を段階的に追加可能

---

## Phase 4: GPU Infrastructure

### 目的

フレームバッファを生成し、背景レイヤーの描画基盤を構築することで、ゲーム画面の表示が可能になる。

### 実装ファイル

#### gpu.hpp

```cpp
// gpu.hpp
#ifndef GPU_HPP
#define GPU_HPP

#include <cstdint>
#include <array>
#include "constants.hpp"

class GPU {
private:
    // フレームバッファ (ARGB8888)
    std::array<uint32_t, 256 * 192> upper_frame_buffer;
    std::array<uint32_t, 256 * 192> lower_frame_buffer;

    // VRAM (ビデオメモリ)
    std::array<uint8_t, 512 * 1024> vram;  // 512KB

    // パレットメモリ (背景とスプライト用)
    std::array<uint16_t, 512> bg_palette;
    std::array<uint16_t, 512> obj_palette;

    // スキャンライン
    int current_scanline;

public:
    GPU();

    // ============================================
    // フレーム更新
    // ============================================

    void update_scanline(int line);
    void render_frame();

    // ============================================
    // フレームバッファアクセス
    // ============================================

    uint32_t* get_upper_frame() { return upper_frame_buffer.data(); }
    uint32_t* get_lower_frame() { return lower_frame_buffer.data(); }

    // ============================================
    // 背景レイヤー描画
    // ============================================

    void draw_background_layer(int layer);
    void render_text_bg(int layer);

    // ============================================
    // VRAM/パレットアクセス
    // ============================================

    uint8_t read_vram(uint32_t addr);
    void write_vram(uint32_t addr, uint8_t value);

private:
    // ============================================
    // 内部ヘルパー関数
    // ============================================

    // RGB555 → RGB888 変換
    uint32_t palette_to_argb(uint16_t palette_entry);

    // タイル描画サブルーチン
    void draw_tile(uint32_t* buffer, int x, int y, int tile_idx, uint16_t* palette);
};

#endif
```

#### gpu.cpp (基本実装)

```cpp
// gpu.cpp
#include "gpu.hpp"
#include <cstring>

GPU::GPU() : current_scanline(0) {
    // フレームバッファを黒で初期化
    upper_frame_buffer.fill(0xFF000000);  // α=255, RGB=0
    lower_frame_buffer.fill(0xFF000000);

    // パレットを初期化
    bg_palette.fill(0x0000);
    obj_palette.fill(0x0000);
}

void GPU::render_frame() {
    // 192本のスキャンラインをレンダリング
    for (int line = 0; line < 192; line++) {
        update_scanline(line);
    }
}

void GPU::update_scanline(int line) {
    current_scanline = line;

    // 背景レイヤーを描画 (BG0-3)
    for (int i = 0; i < 4; i++) {
        draw_background_layer(i);
    }

    // スプライト (OBJ) はPhase 4では簡略化
    // 後で詳細実装
}

uint32_t GPU::palette_to_argb(uint16_t palette_entry) {
    // Nintendo DS のパレットは RGB555形式
    // (ビット配置: XBBBBBGGGGGRRRRR)

    uint8_t r = (palette_entry & 0x1F) << 3;       // 赤5ビット → 8ビット
    uint8_t g = ((palette_entry >> 5) & 0x1F) << 3; // 緑5ビット → 8ビット
    uint8_t b = ((palette_entry >> 10) & 0x1F) << 3; // 青5ビット → 8ビット

    // ARGB8888形式で返す
    return 0xFF000000 | (b << 16) | (g << 8) | r;
}

void GPU::draw_background_layer(int layer) {
    // 簡単な実装: 単色で埋める
    // Phase 4では複雑な背景処理は不要

    uint32_t* target = (current_scanline < 192) ?
        upper_frame_buffer.data() : lower_frame_buffer.data();

    int line_idx = current_scanline % 192;
    uint32_t color = (layer % 4) * 0x40404040;  // レイヤーごと異なる色

    std::fill(target + line_idx * 256,
              target + (line_idx + 1) * 256, color);
}
```

### なぜこの実装か？

1. **ARGB8888 フォーマット**

   - 最も一般的な 32 ビット色フォーマット
   - ディスプレイへの出力が直接可能
   - RGB555 パレットから変換する必要がある

2. **VRAM の独立管理**

   - CPU メインメモリとは別のメモリ領域
   - GPU 専用アクセスパターンにより高速化可能

3. **スキャンライン単位の処理**
   - GPU は 1 本ずつスキャンラインを生成
   - V-BLANK 割り込みと同期（Phase 8）

---

## Phase 5: BIOS / ROM Loading

### 目的

BIOS とゲーム ROM を正しくメモリにロードし、エミュレータがゲームを起動できるようにする。

### 実装ファイル

#### cartridge.hpp

```cpp
// cartridge.hpp
#ifndef CARTRIDGE_HPP
#define CARTRIDGE_HPP

#include <cstdint>
#include <string>
#include <vector>

struct ROMHeader {
    char game_title[12];
    char game_code[4];
    uint16_t maker_code;
    uint8_t unit_code;
    uint8_t device_type;
    uint8_t device_size;
    // ...
};

class Cartridge {
private:
    std::vector<uint8_t> rom_data;
    ROMHeader header;
    bool loaded;

public:
    Cartridge();

    // ============================================
    // ROM ロード
    // ============================================

    bool load_from_file(const std::string& path);

    // ============================================
    // ヘッダ解析
    // ============================================

    void parse_header();
    std::string get_game_title() const;
    uint32_t get_rom_size() const;

    // ============================================
    // ROM メモリアクセス
    // ============================================

    uint8_t read8(uint32_t offset);
    uint32_t read32(uint32_t offset);

    bool is_loaded() const { return loaded; }
};

#endif
```

#### boot_sequence.cpp

```cpp
// boot_sequence.cpp - ブート処理

#include "boot_sequence.hpp"
#include "emulator.hpp"
#include "memory.hpp"
#include <iostream>

bool BootSequence::boot_from_bios(Emulator* emu, const std::string& bios9_path,
                                   const std::string& bios7_path) {
    // 1. BIOS イメージをメモリにロード
    Memory* mem = emu->get_memory();

    std::ifstream bios9_file(bios9_path, std::ios::binary);
    if (!bios9_file) {
        std::cerr << "Failed to load ARM9 BIOS" << std::endl;
        return false;
    }
    std::vector<uint8_t> bios9_data(4096);
    bios9_file.read((char*)bios9_data.data(), 4096);

    // メモリにBIOSをロード
    for (int i = 0; i < 4096; i++) {
        mem->write8(0x00000000 + i, bios9_data[i]);
    }

    // 2. ARM9を初期化
    ARM_CPU* arm9 = emu->get_arm9();
    arm9->power_on();

    // 3. ゲーム実行開始
    std::cout << "Boot sequence complete" << std::endl;
    return true;
}

bool BootSequence::boot_directly_to_game(Emulator* emu, const std::string& rom_path) {
    // ROM をロード
    Cartridge* cart = emu->get_cartridge();
    if (!cart->load_from_file(rom_path)) {
        std::cerr << "Failed to load ROM: " << rom_path << std::endl;
        return false;
    }

    // ARM9 のプログラムカウンタをROM開始位置に設定
    ARM_CPU* arm9 = emu->get_arm9();
    arm9->set_pc(0x08000000);  // ROM start address
    arm9->power_on();

    std::cout << "Booting directly to game: " << cart->get_game_title() << std::endl;
    return true;
}
```

### なぜこの実装か？

1. **ROM ヘッダの解析**

   - ゲーム情報（タイトル、メーカーコード）を取得
   - セーブファイルタイプ判定（後の Phase で使用）

2. **2 つのブート方式**

   - BIOS から起動: 正確だが、BIOS ファイルが必要
   - 直接ゲーム起動: FreeBIOS でも対応可能

3. **段階的なローディング**
   - BIOS をロード → CPU を初期化 → ゲーム実行
   - 各段階でエラーチェック

---

（以下、Phase 6-14 も同様の構造で実装解説）

## Phase 6: UI / Threading

### 目的

Qt フレームワークを使用してグラフィカルなユーザーインターフェースを実装し、
マルチスレッド処理でエミュレーションを流暢に実行できるようにする。

### 実装ファイル

#### emuwindow.hpp

```cpp
// emuwindow.hpp
#ifndef EMUWINDOW_HPP
#define EMUWINDOW_HPP

#include <QMainWindow>
#include <QLabel>
#include "emuthread.hpp"

class EmuWindow : public QMainWindow {
    Q_OBJECT

private:
    EmuThread* emu_thread;
    QLabel* display_label;  // フレームバッファ表示用
    QString current_rom_path;

public:
    EmuWindow(QWidget* parent = nullptr);
    ~EmuWindow();

    // ============================================
    // UI イベントハンドラ
    // ============================================

    void closeEvent(QCloseEvent* event) override;
    void keyPressEvent(QKeyEvent* event) override;
    void keyReleaseEvent(QKeyEvent* event) override;

public slots:
    void load_rom();
    void on_frame_ready(uint32_t* upper, uint32_t* lower);
    void on_fps_updated(int fps);
    void show_preferences();
};

#endif
```

#### emuthread.hpp

```cpp
// emuthread.hpp - エミュレーション実行スレッド
#ifndef EMUTHREAD_HPP
#define EMUTHREAD_HPP

#include <QThread>
#include <QMutex>
#include "emulator.hpp"

class EmuThread : public QThread {
    Q_OBJECT

private:
    Emulator emulator;
    QMutex pause_mutex;
    QMutex key_mutex;

    bool running;
    bool paused;
    int pause_count;

    uint32_t frame_counter;
    int fps;

protected:
    void run() override;  // スレッドのメイン関数

public:
    EmuThread(QObject* parent = nullptr);
    ~EmuThread();

    // ============================================
    // エミュレーション制御
    // ============================================

    void load_rom(const QString& path);
    void pause();
    void unpause();
    void shutdown();

    // ============================================
    // キー入力
    // ============================================

    void press_key(DS_KEY key);
    void release_key(DS_KEY key);

signals:
    void frame_ready(uint32_t* upper, uint32_t* lower);
    void fps_updated(int fps);
    void load_failed(QString error);
};

#endif
```

#### emuthread.cpp

```cpp
// emuthread.cpp
#include "emuthread.hpp"
#include <QElapsedTimer>
#include <iostream>

EmuThread::EmuThread(QObject* parent)
    : QThread(parent), running(false), paused(false),
      pause_count(0), frame_counter(0), fps(0) {
}

void EmuThread::run() {
    // エミュレーション実行ループ
    running = true;
    QElapsedTimer timer;
    timer.start();
    int frame_count = 0;

    while (running) {
        // ポーズ状態を確認
        {
            QMutexLocker locker(&pause_mutex);
            while (paused && running) {
                // ポーズ中はスリープ
                msleep(10);
            }
        }

        // 1フレーム分のサイクル実行 (67737600 Hz / 60 FPS)
        const int cycles_per_frame = 1129600;

        for (int i = 0; i < cycles_per_frame && running; i++) {
            emulator.execute_cycle();
        }

        // フレームバッファをUIに送信
        emit frame_ready(
            emulator.get_gpu()->get_upper_frame(),
            emulator.get_gpu()->get_lower_frame()
        );

        frame_count++;

        // FPS計算 (1秒ごと)
        if (timer.elapsed() >= 1000) {
            fps = frame_count;
            emit fps_updated(fps);

            frame_count = 0;
            timer.restart();
        }
    }
}

void EmuThread::load_rom(const QString& path) {
    if (!emulator.load_rom(path.toStdString())) {
        emit load_failed("Failed to load ROM");
        return;
    }

    if (!start()) {
        emit load_failed("Failed to start emulation thread");
    }
}

void EmuThread::pause() {
    QMutexLocker locker(&pause_mutex);
    pause_count++;
    paused = (pause_count > 0);
}

void EmuThread::unpause() {
    QMutexLocker locker(&pause_mutex);
    pause_count--;
    paused = (pause_count > 0);
}
```

### なぜこの実装か？

1. **スレッド分離**

   - UI スレッドがブロックされないようエミュレーション処理を別スレッドで実行
   - UI の応答性を保つ

2. **フレーム同期**

   - 1 フレーム (60FPS) ごとにシグナル送信
   - UI のリフレッシュレートと同期

3. **Mutex によるスレッドセーフ**
   - キー入力やポーズ状態を安全に共有
   - データ競合バグを防止

---

（以降の Phase も同様の構造で実装詳細を記載）

## Phase 7: Audio System

**SPU (Sound Processing Unit) の実装**

- 16 チャネルのサウンドミキシング
- PCM/ADPCM デコード
- PortAudio でのオーディオ出力

## Phase 8: Interrupt System

**割り込みコントローラの実装**

- V-BLANK / H-BLANK 割り込み
- タイマー割り込み
- IPC (プロセッサ間通信)

## Phase 9: Instruction Set Completion (ARM9) ⭐ CRITICAL

**ARM 命令セットの完全実装**

- メモリ操作命令 (LDM, STM)
- 乗算命令 (MUL, UMULL など)
- Thumb 命令セット完成

## Phase 10: ARM7 Implementation

**ARM7 補助プロセッサの実装**

- ARM9 と同じ CPU クラスを再利用
- IPC (FIFO) 通信機構
- デュアル CPU 同期

## Phase 11: 3D Graphics

**GPU 3D エンジンの実装**

- ジオメトリエンジン
- テクスチャマッピング
- Z-Buffer による隠面消去

## Phase 12: Save Data System

**セーブデータの管理**

- EEPROM/FLASH メモリエミュレーション
- セーブタイプ自動判定
- ファイル I/O

## Phase 13: WiFi / Networking

**ワイヤレス通信機能**

- WiFi MAC エミュレーション
- パケット処理
- オンラインゲーム対応

## Phase 14: Optimization / Debugging

**パフォーマンス最適化とデバッガ**

- JIT コンパイル (オプション)
- デバッガ UI (ブレークポイント, レジスタ表示)
- 互換性改善

---

## 🎯 実装上の重要ポイント

### 1. **段階的なテスト**

各フェーズ完了時にテストを実施：

```cpp
// 例: Phase 2 CPU命令テスト
void test_arm_cpu() {
    Memory mem;
    Emulator emu(&mem);
    ARM_CPU cpu(&emu, 0);

    // ADD R0, R1, R2 の実行をテスト
    cpu.set_register(1, 5);
    cpu.set_register(2, 3);
    cpu.execute_arm_instruction(0xE0800002);  // ADD命令コード

    assert(cpu.get_register(0) == 8);  // 5 + 3 = 8
    std::cout << "CPU ADD test PASS" << std::endl;
}
```

### 2. **定数・マジックナンバーの排除**

constants.hpp に集中管理：

```cpp
// ❌ 悪い例
uint8_t val = mem.read8(0x04000000);  // マジックナンバー

// ✅ 良い例
uint8_t val = mem.read8(ARM9::IO_START);  // 意味が明確
```

### 3. **メモリアドレスのバリデーション**

```cpp
bool is_valid_address(uint32_t addr) {
    return (addr >= ARM9::BIOS_START && addr < ARM9::BIOS_START + ARM9::BIOS_SIZE)
        || (addr >= ARM9::MAIN_RAM_START && addr < ARM9::MAIN_RAM_START + ARM9::MAIN_RAM_SIZE)
        // ...
}
```

### 4. **パフォーマンス計測**

```cpp
#include <chrono>

void benchmark_cpu_execution() {
    auto start = std::chrono::high_resolution_clock::now();

    for (int i = 0; i < 1000000; i++) {
        cpu.execute_cycle();
    }

    auto end = std::chrono::high_resolution_clock::now();
    auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);

    std::cout << "1M cycles in " << duration.count() << " ms" << std::endl;
    // 約15ms が目安 (リアルタイム実行速度)
}
```

---

## 🚀 実装開始のチェックリスト

### Phase 1-3 (基盤)

- [ ] constants.hpp 定義完了
- [ ] Memory クラス実装完了
- [ ] CPU クラスの基本構造完了
- [ ] I/O レジスタハンドラ実装完了

### Phase 4-5 (表示・起動)

- [ ] GPU フレームバッファ生成確認
- [ ] BIOS/ROM ロード動作確認
- [ ] ゲーム初期画面表示確認

### Phase 6-8 (UI・割り込み)

- [ ] Qt UI 実装完了
- [ ] マルチスレッド正常動作
- [ ] V-BLANK 割り込み動作確認

### Phase 9 (命令セット) ⭐ CRITICAL

- [ ] ARM 命令カバレッジ 95% 達成
- [ ] Thumb 命令セット実装完了
- [ ] 簡単なゲーム起動確認

### Phase 10-14 (拡張・最適化)

- [ ] ARM7 実装完了
- [ ] 3D グラフィックス対応
- [ ] セーブ機能動作確認
- [ ] 60FPS 安定実行確認

---

このガイドに従って段階的に実装することで、
堅牢で保守性の高い Nintendo DS エミュレータを開発できます。

最も重要なのは **各フェーズでの確実な実装と テスト** です。
先に進む前に必ず現在のフェーズを完成させてください。
