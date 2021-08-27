#!/usr/bin/env bash
#
# BSD 3-Clause License
#
# Copyright (c) 2021, Alex Katlein
# All rights reserved.
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions are met:
#
# 1. Redistributions of source code must retain the above copyright notice, this
#    list of conditions and the following disclaimer.
#
# 2. Redistributions in binary form must reproduce the above copyright notice,
#    this list of conditions and the following disclaimer in the documentation
#    and/or other materials provided with the distribution.
#
# 3. Neither the name of the copyright holder nor the names of its
#    contributors may be used to endorse or promote products derived from
#    this software without specific prior written permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
# AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
# IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
# DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
# FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
# DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
# SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
# CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
# OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
#

export PROFILE=+nightly

export LIBDIR=$(rustc ${PROFILE} --print target-libdir)
export TOOLDIR=${LIBDIR}/bin

export RUSTFLAGS="-Z instrument-coverage"
export LLVM_PROFILE_FILE="architect-rs.profraw"

cargo ${PROFILE} test --package architect-rs --bin architect

export PROFDATA_FILE="architect-rs.profdata"

"${TOOLDIR}/llvm-profdata" merge -sparse ${LLVM_PROFILE_FILE} -o ${PROFDATA_FILE}

export OBJECT_NAME=$(ls target/debug/deps | grep -E -e 'architect-[a-f0-9]+$')

mkdir -p coverage

"${TOOLDIR}/llvm-cov" export \
  -format=lcov \
  -ignore-filename-regex='/.cargo/registry|/library/std' \
  -instr-profile=${PROFDATA_FILE} \
  "target/debug/deps/${OBJECT_NAME}" > coverage/coverage.lcov
