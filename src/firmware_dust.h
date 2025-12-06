/*
  SPDX-FileCopyrightText: (C) 2007 kelpsyberry
  SPDX-License-Identifier: GPL-3.0-or-later
  https://github.com/kelpsyberry/dust/blob/main/core/src/spi/firmware.rs#L8
*/
#ifndef DEFAULT_FIRMWARE_H
#define DEFAULT_FIRMWARE_H

#include <cstdint>
#include <vector>

namespace dust{
uint16_t crc16(uint16_t crc, const uint8_t *data, size_t len);
enum class Model { Ds, Lite, Dsi, Ique, IqueLite };
std::vector<uint8_t> default_firmware(Model model);
}

#endif // DEFAULT_FIRMWARE_H
