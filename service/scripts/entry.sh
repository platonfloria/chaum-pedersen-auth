#!/usr/bin/env bash
set -e

case "$1" in
  app )
    echo 'entry.sh: Running in app mode'
    ./chaum-pedersen-auth
    ;;
  test )
    echo 'entry.sh: Running in Test mode'
    ;;
  * )
    "$@"
    ;;
esac
