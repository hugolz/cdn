#!/bin/sh

mode=release # debug, release

echo Compiling back
if [ "$mode" = release ]
then
  cargo build -p back --release 
else
  cargo build -p back
fi