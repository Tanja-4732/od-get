# od-get

[![dependency status](https://deps.rs/crate/od-get/0.1.0/status.svg)](https://deps.rs/crate/od-get/0.1.0)

A Rust tool for recursively crawling & downloading data from [open directories](https://www.vice.com/en/article/d35x57/what-are-open-directories)

- [x] Filtering (regex) support
  - [x] Exclude file patterns
  - [x] Include file patterns
  - [x] Exclude folder patterns
  - [x] Include folder patterns
- [x] Customizable output
  - [x] Target directory
  - [ ] Verbosity
  - [ ] JSON file generation
  - [ ] Log file/dynamic terminal output
- [ ] Customizable limits
  - [ ] recursion depth limit
  - [ ] file count limit
  - [ ] file count offset (skip `n` files)
- [x] Multi threaded (using `rayon`)

(work in progress, one layer of recursion works)

## Licence & Copyright

Copyright (c) 2021 Bernd-L. All rights reserved.

![AGPL v3: Free as in Freedom](https://www.gnu.org/graphics/agplv3-with-text-162x68.png)

od-get is free software: you can redistribute it and/or modify it under the terms of the [GNU Affero General Public License](/LICENSE.md) as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

od-get is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the [GNU Affero General Public License](/LICENSE.md) for more details.

You should have received a copy of the [GNU Affero General Public License](/LICENSE.md) along with od-get. If not, see <https://www.gnu.org/licenses/>.

This project (including its source code and its documentation) is released under the terms of the [GNU Affero General Public License](/LICENSE.md).
