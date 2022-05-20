# ATM emulator

The code works as requested in the instructions and it is commented accordingly.

How to run:
```bash
cargo run --release -- input.txt
```

NOTES:
- The solution is done by implementing asynchronous tasks, this way it gets closer to a real case scenario. There are 3 asynchronous tasks: reader, processors and writer.
- The tasks communicate between each other through channels
- The errors are handled poorly, and are ignored: a proper logging system should be implemeneted
- A command must be exactly as specified, otherwise it will be ignored (e.g. dispute command with an amount attached is ignored)
- All type of errors are ignored, meaning that the program does not panic, but on the other hand, it does nothing to log it or report it (room for improvement)
- The code is commented accordingly
- The code is unit tested, in order to run the tests:
```bash
cargo test
```
