// SPDX-License-Identifier: GPL-2.0-or-later
// OpenSCAD worker extension. Not part of the MIT Rust application.
#pragma once
#include <map>
#include <string>
#include <vector>
#include "core/AST.h"

struct ScadstudioFrame {
  size_t event;
  std::string call;
  Location source;
  std::map<std::string, std::string> context;
};
