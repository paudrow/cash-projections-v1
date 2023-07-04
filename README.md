README
======

See how your cashflow looks over time.

This project looks at a CSV file that specifies one-off or reoccuring cash events and then combines them on a month by month basis.

```bash
$ cash-projections-v1 % cargo run -- -m 6
   Compiling cash-projections-v1 v0.1.0 (...)
     Running `target/debug/cash-projections-v1 -m 6`
2023-07:           7192.33      ==>        7192.33
2023-08:           7192.33      ==>       14384.66
2023-09:           7192.33      ==>       21576.98
2023-10:           7192.33      ==>       28769.31
2023-11:           7192.33      ==>       35961.64
2023-12:          17451.02      ==>       53412.66
2024-01:         109784.26      ==>      163196.93
```

Setup
-----

To use this project, make sure that you have [Rust](https://www.rust-lang.org/tools/install) installed.

Then you can run `cargo run` to run the project, or `cargo run -- --help` to see the help options.

An example CSV file with expenses can be found at in [`data`](./data/cash_events.csv).
You can put your own numbers in here.
