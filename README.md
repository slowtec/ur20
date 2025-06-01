# UR20

`ur20` is a [Rust](https://rust-lang.org) library for the
[Weidmüller](http://www.weidmueller.de) u-remote IP20 system.

[![Crates.io version](https://img.shields.io/crates/v/ur20.svg)](https://crates.io/crates/ur20)
[![Docs](https://docs.rs/ur20/badge.svg)](https://docs.rs/ur20/)

## Compatibility

This crate is far from having support for all available modules.
If any of the modules in the system are unsupported, this will panic upon setup.

## Contributing

To add support for new modules, you must

1. Add a rust module (or extend a generic one)
2. Make the necessary changes to input and output decoding, as well as parameter decoding.
3. Add tests based on the specification.
4. Add a case for the new type in  `ur20_fbc_mod_tcp::Coupler::new`.
5. Add a case in `ur20_fbc_mod_tcp::ModbusParameterRegisterCount::param_register_count`.


## License

Copyright (c) 2017 - 2025, slowtec GmbH

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.
