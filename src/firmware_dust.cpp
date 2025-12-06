/*
  SPDX-FileCopyrightText: (C) 2007 kelpsyberry
  SPDX-License-Identifier: GPL-3.0-or-later
  https://github.com/kelpsyberry/dust/blob/main/core/src/spi/firmware.rs#L8
*/
#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <vector>

namespace dust {
// ------------------- CRC16 計算 -------------------
uint16_t crc16(uint16_t crc, const uint8_t *data, size_t len) {
  for (size_t i = 0; i < len; ++i) {
    crc ^= data[i];
    for (int j = 0; j < 8; ++j) {
      if (crc & 1)
        crc = (crc >> 1) ^ 0xA001; // 一般的な CRC16-IBM
      else
        crc = (crc >> 1);
    }
  }
  return crc;
}

// ------------------- Model enum -------------------
enum class Model { Ds, Lite, Dsi, Ique, IqueLite };

// ------------------- Firmware 作成 -------------------
std::vector<uint8_t> default_firmware(Model model) {
  size_t len;
  switch (model) {
  case Model::Dsi:
    len = 0x2'0000;
    break;
  case Model::Ds:
  case Model::Lite:
    len = 0x4'0000;
    break;
  case Model::Ique:
  case Model::IqueLite:
    len = 0x8'0000;
    break;
  default:
    len = 0x4'0000;
    break;
  }

  std::vector<uint8_t> firmware(len, 0);

  // 0x04 と 0x06 書き込み
  firmware[0x04] = 0x00;
  firmware[0x05] = 0xDB;
  firmware[0x06] = 0x1F;
  firmware[0x07] = 0x0F;

  // "MACh"
  firmware[0x08] = 'M';
  firmware[0x09] = 'A';
  firmware[0x0A] = 'C';
  firmware[0x0B] = 0x68;

  // 0x14 書き込み (len >> 17 << 12)
  uint16_t val14 = static_cast<uint16_t>((len >> 17) << 12);
  firmware[0x14] = val14 & 0xFF;
  firmware[0x15] = (val14 >> 8) & 0xFF;

  // 0x18..0x1C 書き込み
  uint8_t arr18[] = {0x00, 0x00, 0x01, 0x01, 0x06};
  std::copy(std::begin(arr18), std::end(arr18), firmware.begin() + 0x18);

  // 0x1D
  switch (model) {
  case Model::Ds:
    firmware[0x1D] = 0xFF;
    break;
  case Model::Lite:
    firmware[0x1D] = 0x20;
    break;
  case Model::Ique:
    firmware[0x1D] = 0x57;
    break;
  case Model::IqueLite:
    firmware[0x1D] = 0x43;
    break;
  case Model::Dsi:
    firmware[0x1D] = 0x63;
    break;
  }

  // 0x1E
  firmware[0x1E] = 0xFF;
  firmware[0x1F] = 0xFF;

  // 0x20..0x28 書き込み
  uint16_t values[] = {static_cast<uint16_t>((len - 0x200) >> 3), 0x0B51,
                       0x0DB3, 0x4F5D, 0xFFFF};
  for (int i = 0; i < 5; ++i) {
    firmware[0x20 + i * 2] = values[i] & 0xFF;
    firmware[0x21 + i * 2] = (values[i] >> 8) & 0xFF;
  }

  // ------------------- user_settings -------------------
  for (int u = 0; u < 2; ++u) {
    size_t start = len - 0x200 + u * 0x100;
    uint8_t *user = firmware.data() + start;

    // 初期値例
    user[0x00] = 5;
    user[0x02] = (u == 0) ? 1 : 0;
    user[0x03] = 1;
    user[0x04] = 1;

    const char *name = "Dust";
    for (int i = 0; i < 4; ++i) {
      user[0x06 + i * 2] = name[i];
      user[0x07 + i * 2] = 0x00;
    }

    // CRC 計算
    uint16_t crc = crc16(0xFFFF, user, 0x70);
    user[0x72] = crc & 0xFF;
    user[0x73] = (crc >> 8) & 0xFF;
  }

  return firmware;
}

// ------------------- main でテスト -------------------
int main() {
  Model model = Model::Ds;

  auto fw = default_firmware(model);

  // ファイルに書き出し
  std::ofstream ofs("firmware_dust.bin", std::ios::binary);
  if (!ofs) {
    std::cerr << "Failed to create file\n";
    return 1;
  }

  ofs.write(reinterpret_cast<const char *>(fw.data()), fw.size());
  ofs.close();

  std::cout << "Saved firmware_dust.bin (" << fw.size() << " bytes)\n";

  // 簡単なテスト: 最初の数バイトを確認
  assert(fw[0x08] == 'M');
  assert(fw[0x09] == 'A');
  assert(fw[0x0A] == 'C');
  assert(fw[0x0B] == 0x68);

  std::cout << "Basic sanity check passed.\n";

  return 0;
}
} // namespace dust