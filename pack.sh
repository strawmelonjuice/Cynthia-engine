3#!/usr/bin/env bash
echo "Packing to 'clean-cyn.tar.gz'..."
cd ./clean_slate
ls -A ./
tar -czf ../clean-cyn.tar.gz * .env