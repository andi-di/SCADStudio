#!/usr/bin/env python3
"""Apply the small GPL worker overlay to an otherwise pristine locked source tree."""
import pathlib
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
PIN = "9ba0cee7c94785fac0ca8dfe040b965a12a25ed1"


def replace(path, old, new):
    text = path.read_text()
    if text.count(old) != 1:
        raise RuntimeError(f"Expected exactly one patch anchor in {path}")
    path.write_text(text.replace(old, new))


def main():
    source = pathlib.Path(sys.argv[1]).resolve()
    actual = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
    if actual != PIN:
        raise RuntimeError(f"Wrong OpenSCAD revision: {actual}")
    if subprocess.check_output(["git", "-C", str(source), "status", "--porcelain"], text=True).strip():
        raise RuntimeError("Worker source must be pristine; use a separate checkout")
    for name in ("scadstudio_trace.h", "scadstudio_export.h"):
        shutil.copyfile(ROOT / "engine/overlay" / name, source / "src/core" / name)
    replace(source / "src/core/EvaluationSession.h", '#include "core/AST.h"',
            '#include "core/AST.h"\n#include "core/scadstudio_trace.h"\n#include <cstdlib>')
    replace(source / "src/core/EvaluationSession.h", "  size_t push_frame(ContextFrame *frame);", """  // Per-evaluation state: no process-global mutable trace stack.
  bool scadstudioEnabled = std::getenv("SCADSTUDIO_TRACE_DIR") != nullptr;
  size_t scadstudioNext = 0;
  std::vector<ScadstudioFrame> scadstudioStack;
  size_t push_frame(ContextFrame *frame);""")
    replace(source / "src/core/ContextFrame.h", '#include "core/AST.h"',
            '#include "core/AST.h"\n#include "core/scadstudio_trace.h"')
    replace(source / "src/core/ContextFrame.h", "  EvaluationSession *session() const", """  std::map<std::string, std::string> scadstudioVariables() const {
    std::map<std::string, std::string> result;
    for (const auto& entry : lexical_variables) result[entry.first] = entry.second.toString();
    for (const auto& entry : config_variables) result[entry.first] = entry.second.toString();
    return result;
  }
  EvaluationSession *session() const""")
    replace(source / "src/core/node.h", '#include "core/AST.h"',
            '#include "core/AST.h"\n#include "core/scadstudio_trace.h"')
    replace(source / "src/core/node.h", "  const ModuleInstantiation *modinst;",
            "  const ModuleInstantiation *modinst;\n  std::vector<ScadstudioFrame> scadstudioOrigin;")
    replace(source / "src/core/ModuleInstantiation.cc", '#include "core/Context.h"',
            '#include "core/Context.h"\n#include "core/node.h"')
    replace(source / "src/core/ModuleInstantiation.cc", "  try {\n    auto node = module->module->instantiate", """  auto session = context->session();
  struct TraceGuard {
    EvaluationSession *session;
    ~TraceGuard() { if (session->scadstudioEnabled) session->scadstudioStack.pop_back(); }
  };
  if (session->scadstudioEnabled) {
    std::map<std::string, std::string> variables;
    // Traverse nearest-to-farthest so lexical shadowing remains visible correctly.
    for (auto current = context; current; current = current->getParent()) {
      auto locals = current->scadstudioVariables();
      for (auto it = locals.begin(); it != locals.end(); ++it)
        if (!variables.count(it->first)) variables[it->first] = it->second;
    }
    session->scadstudioStack.push_back({++session->scadstudioNext,
      this->name(), this->location(), std::move(variables)});
  }
  TraceGuard guard{session};
  try {
    auto node = module->module->instantiate""")
    replace(source / "src/core/ModuleInstantiation.cc", "    return node;", """    if (node && session->scadstudioEnabled) node->scadstudioOrigin = session->scadstudioStack;
    return node;""")
    replace(source / "src/openscad.cc", '#include "core/Context.h"',
            '#include "core/Context.h"\n#include "core/scadstudio_export.h"')
    replace(source / "src/openscad.cc", 'int openscad_main(int argc, char **argv)\n{',
            'int openscad_main(int argc, char **argv)\n{\n  if (argc == 2 && std::string(argv[1]) == "--scadstudio-capabilities") {\n    std::cout << scadstudio_capabilities().dump() << std::endl; return 0;\n  }')
    replace(source / "src/openscad.cc", '  Tree tree(root_node, fparent.string());',
            '  Tree tree(root_node, fparent.string());\n  scadstudio_export(absolute_root_node, tree);')
    replace(source / "CMakeLists.txt", 'find_package(PkgConfig)',
            'find_package(nlohmann_json 3.10 REQUIRED)\nfind_package(PkgConfig)')


if __name__ == "__main__":
    main()
