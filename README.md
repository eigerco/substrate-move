[![License](https://img.shields.io/badge/license-Apache-green.svg)](LICENSE)

# MoveVM for Substrate

This is a modified MoveVM fork for the use of MoveVM in the [pallet-move] Substrate repo.
Please check the [pallet-move] repository to learn more about this project.

## Requirements

`smove` is a package manager for Move language in Substrate. Follow the [instructions](https://github.com/eigerco/smove) to install it.

## Development

For the initial development setup, run the script:
```sh
./scripts/dev_setup.sh -ypt
```

### Design

[pallet-move] uses the `move-vm-backend` to interact with the MoveVM.

The integral part of MoveVM functionality still lies within the `language` directory and contains only the necessary modifications which make the MoveVM operable within the Substrate framework.

### Testing

To run tests for the MoveVM implementation, execute:
```sh
cargo test
```

To run tests for the `move-vm-backend` implementation, execute:
```sh
cargo test -p move-vm-backend --features build-move-projects-for-test # the `backend` main crate
cargo test -p move-vm-backend-common --features build-move-projects-for-test # helper crate for interaction with smove and pallet-move
cargo test -p move-vm-support # helper crate for interaction with language directory
```
_Note: the feature flag `build-move-projects-for-test` needs to be provided only once in order to build all the necessary `move-vm-backend/tests/assets/move-projects/` projects for test purposes (with the `smove-build-all.sh` script). Also, the feature flag needs to be provided whenever any of those Move projects are modified._

## License

Move is licensed as [Apache 2.0](https://github.com/move-language/move/blob/main/LICENSE).

[pallet-move]: https://github.com/eigerco/pallet-move
