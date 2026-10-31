// SPDX-License-Identifier: GPL-2.0-or-later
#pragma once
#include <fstream>
#include <nlohmann/json.hpp>
#include <functional>
#include <stdexcept>
#include "core/scadstudio_trace.h"
#include "core/CsgOpNode.h"
#include "core/TransformNode.h"
#include "geometry/GeometryEvaluator.h"
#include "io/export.h"

inline nlohmann::json scadstudio_location(const Location& loc)
{
  return {{"file", loc.fileName()}, {"first_line", loc.firstLine()},
          {"first_column", loc.firstColumn()}, {"last_line", loc.lastLine()},
          {"last_column", loc.lastColumn()}};
}

inline nlohmann::json scadstudio_capabilities()
{
  return {{"protocol_version", 1}, {"engine_revision", "9ba0cee7c94785fac0ca8dfe040b965a12a25ed1"},
          {"evaluated_origins", true}, {"subtree_geometry", true}, {"egl_png", true},
          {"cooperative_cancel", false}, {"node_limit", 4096}};
}

// Called only by the worker, before normal export. The original tree is never mutated.
inline void scadstudio_export(const std::shared_ptr<const AbstractNode>& root, const Tree& tree)
{
  const auto directory = std::getenv("SCADSTUDIO_TRACE_DIR");
  if (!directory) return;
  const fs::path dir(directory);
  std::ifstream request_stream(dir / "request.json");
  const auto request = nlohmann::json::parse(request_stream);
  if (!fs::is_directory(dir)) throw std::runtime_error("trace directory missing");
  nlohmann::json nodes = nlohmann::json::array();
  nlohmann::json frames = nlohmann::json::object(), contexts = nlohmann::json::object();
  std::map<std::map<std::string, std::string>, size_t> context_ids;
  GeometryEvaluator evaluator(tree);
  uintmax_t artifact_bytes = 0;
  std::function<void(const AbstractNode&, int, const Transform3d&, bool)> visit;
  visit = [&](const AbstractNode& node, int parent, const Transform3d& ancestors, bool subtracted) {
    if (nodes.size() >= 4096) throw std::runtime_error("M0 trace node limit exceeded");
    auto world = ancestors;
    if (const auto transform = dynamic_cast<const TransformNode *>(&node)) world = world * transform->matrix;
    nlohmann::json matrix = nlohmann::json::array();
    for (int row = 0; row < 4; ++row) {
      matrix.push_back({world.matrix()(row, 0), world.matrix()(row, 1),
                        world.matrix()(row, 2), world.matrix()(row, 3)});
    }
    auto origin = nlohmann::json::array();
    for (const auto& frame : node.scadstudioOrigin) {
      origin.push_back(frame.event);
      const auto key = std::to_string(frame.event);
      if (!frames.contains(key)) {
        const auto inserted = context_ids.emplace(frame.context, context_ids.size() + 1);
        const auto context = inserted.first->second;
        if (inserted.second) contexts[std::to_string(context)] = frame.context;
        frames[key] = {{"event", frame.event}, {"call", frame.call},
          {"source", scadstudio_location(frame.source)}, {"context", context}};
      }
    }
    nlohmann::json item = {{"id", node.index()}, {"parent", parent}, {"kind", node.name()},
      {"evaluated", node.toString()}, {"origin", origin},
      {"source", scadstudio_location(node.modinst->location())}, {"world_matrix", matrix},
      {"subtracted", subtracted}, {"mesh", nullptr}};
    // Each subtree is evaluated by the same original GeometryEvaluator. This also
    // retains operands missing from the final difference, with explicit ancestry.
    auto geometry = evaluator.evaluateGeometry(node, true);
    if (geometry && !geometry->isEmpty() && geometry->getDimension() == 3) {
      auto copy = geometry->copy();
      copy->transform(ancestors);
      const auto filename = std::to_string(node.index()) + ".stl";
      std::ofstream stream(dir / filename, std::ios::binary);
      stream.exceptions(std::ios::failbit | std::ios::badbit);
      export_stl(std::shared_ptr<const Geometry>(std::move(copy)), stream, true);
      stream.close();
      const auto bytes = fs::file_size(dir / filename);
      artifact_bytes += bytes;
      if (bytes > 64 * 1024 * 1024 || artifact_bytes > 256 * 1024 * 1024)
        throw std::runtime_error("M0 geometry artifact budget exceeded");
      item["mesh"] = filename;
    }
    nodes.push_back(std::move(item));
    const auto csg = dynamic_cast<const CsgOpNode *>(&node);
    size_t index = 0;
    for (const auto& child : node.getChildren()) {
      visit(*child, node.index(), world,
            subtracted || (csg && csg->type == OpenSCADOperator::DIFFERENCE && index > 0));
      ++index;
    }
  };
  visit(*root, 0, Transform3d::Identity(), false);
  std::ofstream stream(dir / "manifest.json");
  stream.exceptions(std::ios::failbit | std::ios::badbit);
  stream << nlohmann::json{{"protocol_version", 1}, {"snapshot_id", request.at("snapshot_id")},
    {"revision", request.at("revision")}, {"nodes", nodes}, {"frames", frames}, {"contexts", contexts}}.dump(2);
}
