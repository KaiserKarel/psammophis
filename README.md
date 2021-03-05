# Psammophis

[rustpython](https://github.com/rustpython) interpreter with alternative
builtins, stdlib and modules to allow for capability based, sandboxed python execution.

## Implementation

### open

- [ ] implemented
- [x] will-implement

`builtins.open` and `io.OpenWrapper` are mapped to a custom function, which compares
the paths and filemodes with the provided capabilities.

### blacklisted modules

- [ ] implemented
- [x] will-implement

The following modules are not available at all (not included with the distribution):
`os`, `winapi`, `subprocess`, `socket`, `platform` (includes built info) and `asyncio`,

### gutting `http`

- [ ] implemented
- [x] will-implement

As with `open`, most functions in the `http` module are wrapped in custom functions.
  
