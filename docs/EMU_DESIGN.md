# Nintendo DS エミュレータ - ゼロから実装する設計ガイド

## 概要

本ドキュメントは、Nintendo DS エミュレータをゼロから実装するための**段階的な設計と実装順序**を提案します。
CorgiDS をベースに、最小限の機能から始めて段階的に機能を追加していくアプローチを取ります。

---

## 📋 実装計画チェックリスト

- [x] **フェーズ 0**: FreeBIOS 開発（1〜2 週）
- [ ] **フェーズ 1**: 基盤構築（メモリ、定数定義）（1〜2 週）
- [ ] **フェーズ 2**: CPU コア実装（ARM9）（2〜3 週）
- [ ] **フェーズ 3**: メモリ / I/O 管理（2 週）
- [ ] **フェーズ 4**: GPU 基盤構築（2〜3 週）
- [ ] **フェーズ 5**: BIOS / ROM ロード（1〜2 週）
- [ ] **フェーズ 6**: UI / スレッド処理（2 週）
- [ ] **フェーズ 7**: オーディオシステム（1〜2 週）
- [ ] **フェーズ 8**: 割り込みシステム（2 週）
- [ ] **フェーズ 9**: 命令セット完成（ARM9）（3〜4 週）
- [ ] **フェーズ 10**: ARM7 実装（2〜3 週）
- [ ] **フェーズ 11**: 3D グラフィックス（3〜4 週）
- [ ] **フェーズ 12**: セーブデータシステム（1〜2 週）
- [ ] **フェーズ 13**: WiFi / ネットワーク（2〜3 週）
- [ ] **フェーズ 14**: 最適化 / デバッグ（随時）

---

## Phase 0: FreeBIOS Development (完了 ✅)

### 目標

フリーソフトウェアベースの BIOS エミュレーション実装

フリーおよびオープンソースの BIOS エミュレーションを構築することで、
ユーザーが BIOS ファイルをダンプせずにエミュレータを使用できるようにします。

**実装項目:**

- ARM9/ARM7 BIOS の基本的な初期化ルーチン
- ブート シーケンス
- システム メモリ初期化
- 割り込みハンドラの基本実装

**利点:**
✅ ユーザーが物理ハードウェアから BIOS をダンプする必要なし
✅ オープンソースで完全に独立した実装
✅ 法的問題なし

---

## Phase 1: Foundation Setup - Memory & Constants (1-2 週間)

### 目標

プロジェクト基盤、ビルドシステム、基本的なメモリレイアウトの実装

### 1.1 プロジェクト構成の決定

```
emulator/
├── src/
│   ├── core/
│   │   ├── memory.hpp/cpp
│   │   └── constants.hpp
│   ├── cpu/
│   ├── gpu/
│   ├── audio/
│   ├── io/
│   └── main.cpp
├── include/
│   └── (ヘッダーファイル)
├── CMakeLists.txt (またはMeson)
└── README.md
```

### 1.2 実装項目

**優先度: 必須**

1. **メモリ構造の定義** (`memory.hpp`)

   - メモリマップ定義
     ```cpp
     // ARM9 メモリマップ
     const uint32_t ARM9_BIOS_START = 0x00000000;
     const uint32_t ARM9_BIOS_SIZE  = 0x1000;      // 4 KB
     const uint32_t MAIN_MEMORY_START = 0x02000000;
     const uint32_t MAIN_MEMORY_SIZE = 0x400000;   // 4 MB
     const uint32_t VRAM_START = 0x06000000;
     const uint32_t VRAM_SIZE = 0x800000;          // 8 MB
     // ... etc
     ```
   - メモリ読み書きインターフェース

     ```cpp
     class Memory {
         uint8_t main_ram[4 * 1024 * 1024];
         uint8_t vram[8 * 1024 * 1024];
         // ... other memory regions

         uint32_t read32(uint32_t addr);
         uint16_t read16(uint32_t addr);
         uint8_t read8(uint32_t addr);
         void write32(uint32_t addr, uint32_t value);
         void write16(uint32_t addr, uint16_t value);
         void write8(uint32_t addr, uint8_t value);
     };
     ```

2. **定数定義** (`constants.hpp`)

   - DS 仕様定数
   - レジスタオフセット
   - 割り込みマスク

3. **ビルドシステム設定**
   - CMakeLists.txt または meson.build
   - コンパイル対象ファイル定義

### 1.3 チェックリスト

- [ ] メモリマップ定義完了
- [ ] 基本的なメモリ RW 関数実装
- [ ] ビルド成功

---

## Phase 2: CPU Core Implementation (2-3 週間)

### 目標

ARM CPU の基本動作を実装（命令実行サイクルの最小実装）

### 2.1 CPU アーキテクチャ定義

```cpp
class ARM_CPU {
private:
    // レジスタ
    uint32_t r[16];          // R0-R15 (R15 = PC)
    uint32_t cpsr;           // Current PSR
    uint32_t spsr_modes[5];  // Mode別SPSR

    // CPU状態
    bool is_thumb;           // Thumbモード
    bool irq_disabled;
    bool fiq_disabled;

public:
    void fetch();
    void decode();
    void execute();
};
```

### 2.2 実装項目

**段階 1: 基本構造**

1. CPU クラス定義

   - レジスタ配列 (R0-R15)
   - CPSR (プログラムステータスレジスタ)
   - PSR mode enum

2. 基本的なレジスタ操作

   ```cpp
   uint32_t get_register(int idx);
   void set_register(int idx, uint32_t val);
   void set_cpsr(uint32_t val);
   bool get_flag_z();  // ゼロフラグ
   bool get_flag_c();  // キャリフラグ
   ```

3. 命令フェッチループ
   ```cpp
   void run_cycle() {
       uint32_t instruction = memory->read32(pc);
       execute_instruction(instruction);
       pc += 4;
   }
   ```

**段階 2: 簡単な命令セット実装**

優先度の高い命令 (全体の ~70%)：

- MOV (レジスタ移動)
- ADD/SUB (加減算)
- AND/OR/EOR (論理演算)
- LDR/STR (メモリ読み書き)
- B/BL (分岐)

```cpp
// 命令デコード例
void execute_instruction(uint32_t instr) {
    uint8_t cond = (instr >> 28) & 0xF;
    uint8_t op = (instr >> 26) & 0x3;

    if (!check_condition(cond)) return;

    switch (op) {
        case 0: handle_data_processing(instr); break;
        case 1: handle_load_store(instr); break;
        // ...
    }
}
```

### 2.3 テスト方法

- ROM からの命令フェッチが可能か
- 簡単な算術演算が正しく実行されるか
- レジスタ値が更新されるか

### 2.4 チェックリスト

- [ ] CPU クラス実装
- [ ] レジスタ操作関数完成
- [ ] 基本命令 (MOV, ADD など) 実装
- [ ] 簡単なテストプログラムで動作確認

---

## Phase 3: Memory / I/O Management (2 週間)

### 目標

メモリ読み書きの完全実装と I/O レジスタ処理

### 2.1 メモリマップ完成

```cpp
// メモリ領域の完全な定義
class Memory {
private:
    // ARM9
    uint8_t arm9_bios[0x1000];          // 0x00000000
    uint8_t main_ram[0x400000];         // 0x02000000
    uint8_t vram[0x800000];             // 0x06000000
    uint8_t io_regs[0x10000];           // 0x04000000

    // ARM7
    uint8_t arm7_bios[0x4000];          // 0x00000000
    uint8_t arm7_wram[0x10000];         // 0x03800000
    uint8_t shared_wram[0x8000];        // 0x02800000

public:
    // メモリマップ対応読み書き
    uint32_t read32(uint32_t addr);
    void write32(uint32_t addr, uint32_t val);
};
```

### 2.2 実装項目

1. **I/O レジスタ基盤**
   - I/O レジスタ領域のマッピング
   - レジスタ読み書き処理
2. **キー入力レジスタ** (KEYINPUT)

   ```cpp
   // 0x04000130
   struct KEYINPUT {
       bool a, b, select, start;
       bool right, left, up, down;
       bool r, l;
   };
   ```

3. **割り込みステータスレジスタ** (IE, IF)

   ```cpp
   // 0x04000200, 0x04000202
   struct INTERRUPT {
       uint16_t enable_flags;
       uint16_t interrupt_flags;
   };
   ```

4. **簡単な DMA 実装** (最小限)
   - DMA チャネル定義
   - 基本的なメモリ転送

### 2.3 チェックリスト

- [ ] メモリ領域すべてが正しくマッピングされている
- [ ] I/O レジスタ読み書きが動作
- [ ] キー入力レジスタが機能

---

## Phase 3: GPU (グラフィックス) 基盤 (2-3 週間)

### 目標

フレームバッファ生成と基本的な描画

### 3.1 ディスプレイシステム定義

```cpp
class GPU {
private:
    // フレームバッファ
    uint32_t upper_buffer[256 * 192];  // 上画面
    uint32_t lower_buffer[256 * 192];  // 下画面

    // VRAM
    uint8_t vram[8 * 1024 * 1024];
    uint16_t palette[512];  // BG/OBJパレット

public:
    uint32_t* get_upper_frame();
    uint32_t* get_lower_frame();
    void render_frame();
};
```

### 3.2 実装項目

**段階 1: 最小フレーム生成**

1. **フレームバッファ初期化**

   - ARGB8888 フォーマット (32-bit)
   - 256x192 ピクセル x 2 (上下)

2. **簡単な背景描画**

   - テキストモードの背景レイヤー (BG0-3)
   - パレット色の参照

3. **タイル描画システム**
   ```cpp
   void draw_background_layer(int layer) {
       for (int y = 0; y < 192; y++) {
           for (int x = 0; x < 256; x++) {
               uint8_t tile_idx = get_tile_index(x, y, layer);
               uint32_t pixel = get_tile_pixel(tile_idx, x & 7, y & 7);
               upper_buffer[y * 256 + x] = pixel;
           }
       }
   }
   ```

**段階 2: 段階的な機能追加**

- スプライト (OBJ) 描画
- レイヤー合成
- 回転/スケーリング効果 (拡張 BG)

### 3.3 テスト方法

- BIOS ブートスクリーン表示
- 単色画面表示 (背景色)
- 簡単なパターン描画

### 3.4 チェックリスト

- [ ] フレームバッファ生成可能
- [ ] 簡単な背景描画可能
- [ ] 画面に何らかの画像が表示される

---

## Phase 4: BIOS と ROM ロード (1-2 週間)

### 目標

BIOS/Firmware ロードと ROM のメモリマッピング

### 4.1 実装項目

1. **BIOS/Firmware ロード**

   ```cpp
   bool load_bios(const std::string& arm9_path,
                  const std::string& arm7_path) {
       std::ifstream file(arm9_path, std::ios::binary);
       file.read((char*)arm9_bios, 0x1000);
       // ... ARM7 BIOS も同様
   }
   ```

2. **ROM ヘッダ解析**

   - ゲームタイトル
   - ROM サイズ
   - メモリマップ情報

3. **ROM メモリマップ**

   ```cpp
   // 0x08000000: ROM 領域
   const uint32_t ROM_START = 0x08000000;

   class Cartridge {
       uint8_t rom_data[];
       uint32_t rom_size;

       uint8_t read8(uint32_t addr);
       void load_from_file(const std::string& path);
   };
   ```

### 4.2 チェックリスト

- [ ] BIOS が正しくメモリにロード
- [ ] ROM ヘッダ解析完了
- [ ] ROM からのデータ読み取り可能

---

## Phase 5: UI とスレッド処理 (2 週間)

### 目標

GUI フレームワークの統合とマルチスレッド実装

### 5.1 実装項目

1. **UI フレームワーク選択**

   - Qt (推奨): クロスプラットフォーム
   - SDL2: シンプル
   - ImGui: 軽量

2. **メインウィンドウ**

   ```cpp
   class EmuWindow : public QMainWindow {
   private:
       EmuThread* emu_thread;
       QLabel* screen_display;

   public:
       void display_frame(uint32_t* upper, uint32_t* lower);
       void handle_key_input(Qt::Key key);
   };
   ```

3. **エミュレーションスレッド**

   ```cpp
   class EmuThread : public QThread {
   private:
       Emulator emulator;

   protected:
       void run() override {
           while (running) {
               emulator.execute_cycle();
               if (frame_complete) {
                   emit frame_ready(buffer);
               }
           }
       }
   };
   ```

4. **キー入力マッピング**
   - キーボード → DS ボタン
   - マウス → タッチスクリーン

### 5.2 チェックリスト

- [ ] ウィンドウ表示可能
- [ ] エミュレーション実行スレッド化
- [ ] フレーム表示更新

---

## Phase 6: オーディオ基盤 (1-2 週間)

### 目標

基本的なサウンド処理システム

### 6.1 実装項目

1. **SPU (Sound Processing Unit) 構造**

   ```cpp
   class SPU {
   private:
       struct Channel {
           bool enabled;
           uint16_t volume;
           uint16_t panning;
           uint32_t source_addr;
           uint32_t length;
       } channels[16];

   public:
       void render_audio(int16_t* buffer, int samples);
   };
   ```

2. **簡単なオーディオ出力**

   - PCM デコード (無圧縮)
   - 音量制御
   - パン制御

3. **オーディオバッファリング**
   ```cpp
   // サンプルレート: 32768 Hz
   int16_t audio_buffer[1024];
   void process_audio() {
       spu.render_audio(audio_buffer, 1024);
       output_device.write(audio_buffer, 1024);
   }
   ```

### 6.2 チェックリスト

- [ ] オーディオ出力デバイス初期化
- [ ] 簡単なサウンド生成
- [ ] スピーカーから音が出る

---

## Phase 7: 割り込みシステム (2 週間)

### 目標

割り込み処理と CPU 同期

### 7.1 実装項目

1. **割り込みコントローラ**

   ```cpp
   class InterruptController {
   private:
       uint16_t interrupt_enable;      // IE
       uint16_t interrupt_status;      // IF

       enum {
           VBLANK = 0,
           HBLANK = 1,
           TIMER0 = 2,
           DMA0 = 8,
           // ... etc
       };

   public:
       void set_interrupt(int id);
       void check_interrupts();
   };
   ```

2. **割り込み処理フロー**

   - 割り込みフラグ設定
   - IRQ 割り込みハンドラ呼び出し
   - CPU パイプライン制御

3. **タイマーとタイミング**

   ```cpp
   class Timers {
       struct Timer {
           uint16_t counter;
           uint16_t reload;
           bool enabled;
           uint8_t prescale;
       } timers[4];

       void tick();
   };
   ```

### 7.2 チェックリスト

- [ ] 割り込みフラグ処理実装
- [ ] IRQ/FIQ ハンドラ実装
- [ ] タイマー割り込み動作

---

## Phase 9: Instruction Set Completion - ARM9 (3-4 週間) ⭐ CRITICAL

### 目標

**ARM9 CPU** の命令セットの完成度向上（カバレッジ ~95%）

⚠️ **このフェーズが最も重要です！**

ARM9 がゲームを実行できるレベルまで完全に機能していることが必須です。
このフェーズを完了してからのみ、他の機能を安定的に検証できます。

### 9.1 優先度別実装リスト

**優先度 A (必須 - 最初に実装):**

- ビット操作命令 (BIC, TST, TEQ, ORR)
- メモリブロック操作 (LDM, STM)
- 条件付き分岐 (B, BL, BX)
- Thumb 命令セット基本部
- PSR 操作 (MRS, MSR)

**優先度 B (重要 - 次に実装):**

- 乗算命令 (MUL, MLA)
- 長い乗算 (SMULL, UMULL)
- メモリスワップ (SWP)
- 例外処理 (SWI, UNDEFINED)

**優先度 C (補助 - 後で実装):**

- コプロセッサ命令
- 半精度計算
- 未定義命令の詳細処理

### 9.2 実装チェックポイント

ARM9 が完全に機能しているかの確認項目：

```cpp
✅ メモリアクセス確認
   - Main RAM (0x02000000-0x02FFFFFF)     読み書き可能
   - VRAM (0x06000000-0x06FFFFFF)         読み書き可能
   - I/O Regs (0x04000000-0x04FFFFFF)     読み書き可能
   - ROM (0x08000000+)                    読み取り可能

✅ CPU制御確認
   - CP15 キャッシュ制御                   動作可能
   - メモリ保護 (MPU)                     動作可能
   - 例外処理                             動作可能

✅ 命令実行確認
   - ARM命令 カバレッジ 90%以上
   - Thumb命令 カバレッジ 85%以上
   - 条件フラグ更新                       正確
```

### 9.3 テスト方法

- 簡単なテスト ROM 作成 (ARM9 のみ)
- メモリ読み書きテスト
- 複数命令の組み合わせテスト
- 実ゲーム(簡単なもの)での動作確認

### 9.4 チェックリスト

- [ ] ARM9 命令カバレッジ 95%以上達成
- [ ] すべてのメモリ領域にアクセス可能
- [ ] キャッシュ制御が正常に動作
- [ ] 例外処理が正常に動作
- [ ] 簡単なゲームが起動可能
- [ ] ゲーム内で基本的な画面表示が可能

---

## Phase 10: ARM7 Implementation (2-3 週間)

### 目標

**ARM7 CPU** (補助プロセッサ) の実装と ARM9 との同期

⚠️ **重要:** ARM7 は補助的な役割です

- オーディオ処理
- WiFi 制御
- RTC/タイマー補助
- 周辺機器制御

**多くのゲームは ARM7 がなくても動作します。**
Phase 9 (ARM9 の完全実装) 後に開始してください。### 10.1 実装項目

1. **ARM7 CPU クラス実装**
   - ARM9 と同じ ARM_CPU クラスを再利用
   - cpu_id フィールド (0=ARM9, 1=ARM7) で区別
2. **ARM7 メモリ領域確保**

   ```cpp
   uint8_t arm7_bios[0x4000];        // 16 KB BIOS
   uint8_t arm7_wram[0x10000];       // 64 KB WRAM
   uint8_t shared_wram[0x8000];      // 32 KB 共有WRAM (ARM9と共用)
   ```

3. **ARM7↔ARM9 通信 (IPC)**

   ```cpp
   class IPC {
       struct FIFO {
           std::queue<uint32_t> data;
           bool empty_flag, full_flag;
       } fifo7, fifo9;

       void transfer_message(uint32_t val);
   };
   ```

4. **デュアル CPU 実行スケジューラ**
   ```cpp
   void Emulator::run_cycle() {
       arm9.execute();  // ARM9 サイクル実行
       arm7.execute();  // ARM7 サイクル実行
       gpu.process();   // GPU処理
       check_interrupts();
   }
   ```

### 10.2 チェックリスト

- [ ] ARM7 実行ループ実装
- [ ] ARM7 メモリ領域確保
- [ ] IPC (FIFO) 通信動作確認
- [ ] デュアル CPU 同期動作
- [ ] ARM9 + ARM7 両方でゲーム実行可能

---

## Phase 11: 3D Graphics (3-4 週間)

### 目標

3D グラフィックスと複雑な描画効果の実装

### 11.1 実装項目

1. **3D ジオメトリエンジン**

   - ポリゴン処理
   - 座標変換 (行列演算)
   - ラスタライザ

2. **3D コマンド処理**

   ```cpp
   void handle_3d_command(uint32_t cmd) {
       switch (cmd & 0xFF) {
           case MTX_MODE: set_matrix_mode(cmd); break;
           case MTX_PUSH: push_matrix(); break;
           case VTXN: add_vertex(cmd); break;
           case END_VTXS: finish_polygon(); break;
       }
   }
   ```

3. **テクスチャマッピング**

   - テクスチャ座標
   - バイリニア補間

4. **Z バッファと隠面消去**

### 11.2 チェックリスト

- [ ] 3D オブジェクト描画可能
- [ ] 多くのゲームで 3D 表示
- [ ] フレームレート良好

---

## Phase 12: Save Data System (1-2 週間)

### 目標

セーブファイルの読み書き機能実装

### 12.1 実装項目

1. **セーブタイプ検出**

   ```cpp
   enum SaveType {
       SAVE_NONE = 0,      // セーブなし
       SAVE_EEPROM_4K,     // EEPROM 4KB
       SAVE_EEPROM_64K,    // EEPROM 64KB
       SAVE_EEPROM_512K,   // EEPROM 512KB
       SAVE_FLASH_2M,      // FLASH 2MB
       SAVE_FLASH_4M       // FLASH 4MB
   };
   ```

2. **セーブファイル I/O**

   ```cpp
   class SaveFile {
       std::string save_path;
       uint8_t save_data[];

   public:
       void load_from_disk();
       void save_to_disk();
   };
   ```

3. **セーブデータベース**
   - AKAIO savelist.bin サポート

### 12.2 チェックリスト

- [ ] セーブファイル読み取り
- [ ] セーブデータ書き込み
- [ ] ゲーム進行保持

---

## Phase 13: WiFi / Networking (2-3 週間)

### 目標

ワイヤレス通信機能 (オプション)

### 13.1 実装項目

1. **WiFi MAC エミュレーション**

   ```cpp
   class WiFi {
       struct AccessPoint {
           char ssid[33];
           uint8_t bssid[6];
           int8_t rssi;
       };

       void scan_networks();
       void connect(const AccessPoint& ap);
   };
   ```

2. **ワイアレス通信制御**
   - RF チップシミュレーション
   - パケット処理

### 13.2 チェックリスト

- [ ] WiFi メニュー表示
- [ ] ネットワーク機能の基本動作

---

## Phase 14: Optimization / Debugging (進行中)

### 目標

パフォーマンス向上とバグフィックス

### 14.1 実装項目

1. **パフォーマンス最適化**

   - JIT コンパイル (オプション)
   - キャッシング戦略
   - ホットパス最適化

2. **デバッガ機能**

   - ブレークポイント
   - ステップ実行
   - メモリ検査

3. **互換性改善**
   - ゲーム固有の問題対応
   - 複雑な命令シーケンス

### 13.2 チェックリスト

- [ ] 60 FPS 達成
- [ ] 一般的なゲーム全て動作
- [ ] クラッシュなし

---

## 📊 実装順序タイムライン

```
Week 1-2     Phase 0: FreeBIOS Development (完了)

Week 1-3     Phase 1: Foundation Setup
             ├─ メモリ構造定義
             ├─ 定数定義
             └─ ビルドシステム構築

Week 4-6     Phase 2: CPU Core Implementation
             ├─ CPU クラス設計
             ├─ 基本命令実装 (MOV, ADD, LDR/STR)
             └─ 命令フェッチループ

Week 7-8     Phase 3: Memory / I/O Management
             ├─ メモリマップ完成
             ├─ I/O レジスタ処理
             └─ DMA基本実装

Week 9-11    Phase 4: GPU Infrastructure
             ├─ フレームバッファ初期化
             ├─ 背景レイヤー描画
             └─ パレット管理

Week 12-13   Phase 5: BIOS / ROM Loading
             ├─ BIOS ロード機能
             ├─ ROM ヘッダ解析
             └─ ROM メモリマッピング

Week 14-15   Phase 6: UI / Threading
             ├─ UI フレームワーク統合
             ├─ エミュレーションスレッド
             └─ キー入力処理

Week 16-17   Phase 7: Audio System
             ├─ SPU構造実装
             ├─ PCM デコード
             └─ オーディオ出力

Week 18-19   Phase 8: Interrupt System
             ├─ 割り込みコントローラ
             ├─ IRQ/FIQ ハンドラ
             └─ タイマー実装

Week 20-23   Phase 9: Instruction Set Completion (ARM9) ⭐ CRITICAL
             ├─ 優先度A命令完成
             ├─ 優先度B命令追加
             ├─ Thumb命令セット
             ├─ メモリアクセス最適化
             └─ ゲーム起動テスト

Week 24-26   Phase 10: ARM7 Implementation
             ├─ ARM7 CPU実装
             ├─ ARM7 メモリ領域
             ├─ IPC (FIFO) 通信
             └─ デュアルCPU同期

Week 27-30   Phase 11: 3D Graphics
             ├─ 3D エンジン実装
             ├─ ポリゴン処理
             ├─ テクスチャマッピング
             └─ Z-buffer処理

Week 31-32   Phase 12: Save Data System
             ├─ セーブタイプ判定
             ├─ EEPROM/FLASH 制御
             └─ セーブデータベース

Week 33-35   Phase 13: WiFi / Networking
             ├─ WiFi MAC エミュレーション
             ├─ パケット処理
             └─ ネットワークテスト

進行中:       Phase 14: Optimization / Debugging
             ├─ パフォーマンス最適化
             ├─ デバッガ機能
             └─ 互換性改善
```

---

---

## 📌 実装での重要なポイント

### 1. テスト駆動開発 (TDD)

各フェーズごとにテストを書く：

```cpp
void test_add_instruction() {
    cpu.set_register(0, 5);
    cpu.set_register(1, 3);
    cpu.execute_add(0, 1);  // R0 = R0 + R1
    assert(cpu.get_register(0) == 8);
}
```

### 2. マイルストーン管理

各フェーズ完了時に：

- テストスイート実行
- パフォーマンス計測
- ドキュメント更新
- リリース版作成

### 3. ドキュメント

各モジュールの設計ドキュメント：

```markdown
# ARM CPU Implementation

## Architecture

- レジスタ配置図
- メモリアクセスパターン

## Instruction Format

- ARM / Thumb 命令形式
- 各命令の実装

## Test Results

- テスト覆度率
- 実行時間計測
```

### 4. パフォーマンス計測

```cpp
auto start = std::chrono::high_resolution_clock::now();
for (int i = 0; i < 1000000; i++) {
    cpu.execute_cycle();
}
auto duration = std::chrono::high_resolution_clock::now() - start;
std::cout << "Speed: " << (1000000 / duration.count()) << " cycles/sec" << std::endl;
```

---

## 🛠️ 推奨ツール・ライブラリ

| 用途                  | 推奨           | 代替案               |
| --------------------- | -------------- | -------------------- |
| **ビルドシステム**    | CMake 3.16+    | Meson, Bazel         |
| **UI フレームワーク** | Qt 5.12+       | SDL2, SFML, ImGui    |
| **オーディオ**        | PortAudio      | OpenAL, SDL2_mixer   |
| **テスティング**      | Google Test    | Catch2, Doctest      |
| **デバッグ**          | GDB/LLDB       | Visual Studio, Xcode |
| **CI/CD**             | GitHub Actions | GitLab CI, Travis CI |
| **バージョン管理**    | Git            | (推奨)               |

---

## 📚 参考リソース

### ARM アーキテクチャ

- ARM Architecture Reference Manual (ARMv5TE)
- ARM Instruction Set Quick Reference Card
- Thumb Instruction Set Reference

### Nintendo DS

- GBATEK - GBA/NDS 仕様完全ガイド
- melonDS GitHub ソースコード
- CowBite Spec (非公式仕様書)

### エミュレータ実装

- "Writing a Game Boy Emulator in Rust" - David Keezer
- "Game Engine Architecture" - Fabien Sanglard
- melonDS, Dolphin, PCSX2 のソースコード研究

### ビデオ・チュートリアル

- Emulator 101 Blog
- low-level learning チャネル

---

## ❓ よくある質問とトラブルシューティング

### Q: CPU が全く実行されない

**A:** 確認項目

- [ ] BIOS がメモリに正しくロードされているか
- [ ] メモリマップが正しいか
- [ ] PC (プログラムカウンタ) が初期値から進むか

### Q: 画面が真っ黒 / 映らない

**A:** 確認項目

- [ ] GPU がフレームバッファを生成しているか
- [ ] V-BLANK 割り込みが発生しているか
- [ ] VRAM から正しくデータを読み取っているか

### Q: サウンドが全く出ない

**A:** 確認項目

- [ ] SPU が初期化されているか
- [ ] オーディオデバイスが正しく初期化されているか
- [ ] SPU タイミングが正確か

### Q: ゲームが起動するがすぐクラッシュ

**A:** デバッガで確認

- [ ] 例外が発生していないか (UND、ABT)
- [ ] メモリアクセス違反がないか
- [ ] 未実装命令を実行していないか

### Q: フレームレートが低い

**A:** 最適化ポイント

- [ ] CPU 命令実行の最適化
- [ ] メモリアクセスパターンの見直し
- [ ] GPU 描画ルーチンの高速化

---

## 🎓 このガイドの使い方

1. **初期計画時**

   - チェックリストを使用してタスク管理
   - Timeline に基づいてスケジュール立案

2. **開発中**

   - 各フェーズのチェックリストで進捗確認
   - 依存関係を確認してブロッカーを排除

3. **テスト・検証時**

   - 各フェーズの目標が達成されているか確認
   - テストケースを実行

4. **リリース時**
   - チェックリスト完了確認
   - リリースノート作成

---

このガイドに従うことで、体系的で保守性の高い Nintendo DS エミュレータを開発できます。
重要なのは **Phase 9 (ARM9 命令セット完成)** を確実に完了させることです。
ここが完了すれば、他の機能は比較的容易に追加できます。
