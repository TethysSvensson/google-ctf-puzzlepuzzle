#!/bin/sh
cargo run --release --bin dat2raw
git checkout shape_db.json
rm -f cached_groups.bin
cargo run --release --bin fix_bits
cargo run --release --bin solve 2214..6611,83639 --step-x 24
cargo run --release --bin solve 6625,83718..90500 --step-y 12
cargo run --release --bin solve 6..6650,90290 --step-x 12
cargo run --release --bin solve 8862,74822..81443 --step-y 12
cargo run --release --bin solve 8857,57162..75145 --step-y 12
cargo run --release --bin solve 6637,83682 8869,74790 8869,57126 8869,39438 17257,8898 17257,8910 8858,34962
cargo run --release --bin solve 30..8890,57083 --step-x 24
cargo run --release --bin solve 30..8890,39395 --step-x 24
cargo run --release --bin solve 8857,39474..52700 --step-y 12
cargo run --release --bin solve 8418..17257,8855 --step-x 24
cargo run --release --bin solve 17245,8934..22200 --step-y 12

cargo run --release --bin solve_trial $(for x in {0..184}; do echo $((6594 - (24 * $x))),83642; done)
mv solution_0.raw puzzlepuzzle.raw
mv cached_groups_0.bin cached_groups.bin

for end in 100 200 300 368; do
  cargo run --release --bin solve_trial $(for x in $(seq 0 $end); do echo $((8850 - (24 * $x))),57086; done)
  mv solution_0.raw puzzlepuzzle.raw
  mv cached_groups_0.bin cached_groups.bin
done

for end in 100 200 300 368; do
  cargo run --release --bin solve_trial $(for x in $(seq 0 $end); do echo $((8850 - (24 * $x))),39394; done)
  mv solution_0.raw puzzlepuzzle.raw
  mv cached_groups_0.bin cached_groups.bin
done

for end in 100 200 300 368; do
  cargo run --release --bin solve_trial $(for x in $(seq 0 $end); do echo $((17238 - (24 * $x))),8866; done)
  mv solution_0.raw puzzlepuzzle.raw
  mv cached_groups_0.bin cached_groups.bin
done

cargo run --release --bin cat_flag

## Gen image
cargo run --release --bin gen_image 0 100000 0 100000
gimp output_image.png &
