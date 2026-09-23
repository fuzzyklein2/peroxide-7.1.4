# @file meson.build
project(
  'peroxide',
  'cpp',
  version : '0.1.0',
  default_options : ['cpp_std=c++23']
)

# Include directory for headers
inc = include_directories('include', 'contrib')

# --- AUTOGEN_SRC_START ---
src_files = [
  'source/audio.cpp',
]
# --- AUTOGEN_SRC_END ---

peroxide = static_library(
  'peroxide',
  src_files,
  include_directories: inc,
)

