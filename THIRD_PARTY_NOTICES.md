# Third party notices

The SAIC login protocol in `src/protocol.rs` and `src/client.rs`, and the Python
reference vectors in `tests/fixtures/login.json`, are based on
[saic-python-client-ng](https://github.com/SAIC-iSmart-API/saic-python-client-ng/tree/8ab0634b292e64f5198d979636e01616ea6af34f),
particularly `api/base.py`, `net/crypto.py`, and `crypto_utils.py`.

Command transport in `src/command.rs`, `src/client.rs`, `src/protocol.rs` and
`tests/fixtures/command.json` follows revision
[`5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e`](https://github.com/SAIC-iSmart-API/saic-python-client-ng/tree/5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e),
particularly `api/base.py`, `api/vehicle/`, `api/vehicle_charging/` and
`net/crypto.py`.

The upstream license follows.

MIT License

Copyright (c) 2024 The SAIC-iSmart-API project

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
