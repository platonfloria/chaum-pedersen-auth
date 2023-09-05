#!/usr/bin/env bash
set -e

case "$1" in
  app )
    echo 'entry.sh: Running in app mode'
    ./scripts/wait-for-it.sh --timeout=0 --strict --host=$SERVICE_HOST --port=$SERVICE_PORT -- ./chaum-pedersen-auth
    ;;
  test )
    echo 'entry.sh: Running in Test mode'
    ;;
  * )
    "$@"
    ;;
esac
