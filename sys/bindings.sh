#!/bin/sh

# can't generate tests, since they are arch-specific :(
bindgen \
    --no-layout-tests \
    --default-enum-style moduleconsts \
    --enable-cxx-namespaces \
    --opaque-type std::.+ \
    --blocklist-function std::.+ \
    --blocklist-var std::.+ \
    --allowlist-item SandiaDecay::.+ \
    --allowlist-item sdecay::.+ \
    --allowlist-type sdecay::.+ \
    --allowlist-item std::string \
    --allowlist-item std::vector.+ \
    --no-derive-debug \
    --no-derive-copy \
    --generate-cstr \
    --use-core \
    --no-doc-comments \
    --output bindings.rs \
    wrapper.hpp \
    -- \
    -xc++ \
    -std=c++17 \
    -I/usr/include \
    -I/usr/local/include \
    -Ivendor
