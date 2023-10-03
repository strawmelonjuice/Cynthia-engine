#!/usr/bin/env bash
DISABLED=1;
if [ $DISABLED = 1 ]; then
    echo "You ran the reset.sh script. It is currently disabled."
elif [ $DISABLED = 0 ]; then
    echo "You ran the reset.sh script. Resetting CynthiaConfig..."
    rm -rf './_cynthia/' './assets/'
    rm './.env' './pack.sh' './GETTINGSTARTED.MD' './pack.sh'
    rm './reset.sh'
fi