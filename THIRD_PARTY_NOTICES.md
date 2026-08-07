# Third-party notices

Clutter's direct clustered-snapshot reader was independently ported to Rust
with reference to the Dart SDK serializer and to the `unflutter` project.

## unflutter

Copyright 2024 The Unflutter Authors. All rights reserved.

Redistribution and use in source and binary forms, with or without modification,
are permitted provided that the following conditions are met:

- Redistributions of source code must retain the above copyright notice, this
  list of conditions and the following disclaimer.
- Redistributions in binary form must reproduce the above copyright notice,
  this list of conditions and the following disclaimer in the documentation
  and/or other materials provided with the distribution.
- Neither the name of the copyright holder nor the names of its contributors
  may be used to endorse or promote products derived from this software without
  specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE LIABLE FOR
ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
(INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON
ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

## Dart SDK and Android resource formats

The snapshot grammar follows the Dart SDK implementation, which is distributed
under a BSD-style license. AAB protobuf XML structures follow Android AAPT's
Apache-2.0-licensed `Resources.proto`.

`vm-oracle/dart-sdk-3.11.4.patch` contains modifications to Dart SDK source
files used to build the optional snapshot analyzer. Those modified portions
remain subject to the Dart SDK's copyright and BSD-style license. Clutter does
not redistribute a Dart SDK checkout or analyzer binary.

## Rust dependencies

The built executable includes third-party Rust crates under their respective
licenses, including Capstone (BSD-3-Clause), clap (MIT OR Apache-2.0), object
(MIT OR Apache-2.0), gimli (MIT OR Apache-2.0), prost (Apache-2.0), rayon
(MIT OR Apache-2.0), serde
(MIT OR Apache-2.0), flate2 (MIT OR Apache-2.0), and zip (MIT).
