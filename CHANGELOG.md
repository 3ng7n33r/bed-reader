# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2023-11-04

### Changed

- (Python) Install now with `pip install bed-reader[samples,sparse]`.
  If you want a version that depends only `numpy` (and for now pandas) use
  `pip install bed-reader`.

- (Python) Added new `read_sparse()` method for creating
  `sympy.sparse.csc_matrix` and `sympy.sparse.csr_matrix`. This
  method can save memory when a *.bed file contains mostly 0's.

- (Rust) Rust methods that returned `Result` now return
  `Result<_,Box<BedErrorPlus>>`. Before, they returned
  `Result<_, BedErrorPlus>`. This saves memory.

## [0.2.27] - 2023-10-29

- (Python) Add support for Python 3.12.