#!/bin/sh
# Compile code and look for SIGSEGV in compiler output.
# For use with comby-reducer.

if [ -s "$1" ]
then
  cat "$1" > src/main.rs
fi

ERR=$(cargo +nightly-2021-01-07 build --release 2>&1)

if [[ $ERR =~ "SIGSEGV" ]]
then
    echo "SIGSEGV"
    exit 139
else
    exit 0
fi
