#!/bin/sh
sed -i 's/dbg!/\/\/dbg!/g' src/main.rs
sed -i 's/\/\/dbg!/dbg!/g' src/main.rs
