#!/usr/bin/env bash
echo "Packing to 'clean-cyn.tar.gz'..."
ls -A ./clean_slate/
tar -czf ./clean-cyn.tar.gz ./clean_slate/* ./clean_slate/.env