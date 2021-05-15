#!/bin/bash

rm target/release/chapter-tracker
cargo build --release
cp target/release/chapter-tracker /media/d/ChapterTracker/
