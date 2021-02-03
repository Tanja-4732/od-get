# od-get

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
