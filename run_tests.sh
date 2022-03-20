#!/bin/zsh

BASEDIR=$(pwd)

echo "Building Rust Projects"
for project in test_json-aws_smithy_json test_json-rust test_json-rust-serde_json/rj
do
    cd $BASEDIR/parsers/$project/
    cargo build --release
done

cd $BASEDIR

python3 run_tests.py


