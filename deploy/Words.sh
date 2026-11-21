#!/bin/sh
# Name: Words
# Author: Primitive AI
# DontUseFBInk
# PrimitiveUSBLease
# Wake: redraw

# Under the launcher SIGUSR1 asks the app to redraw: this script ignores it (a shell ends on it otherwise); the
# app sets its own handler.
[ "${PRIMITIVE_LAUNCHER:-0}" = 1 ] && trap '' USR1
app=/mnt/us/words/bin/words-kindle
[ -x "$app" ] || exit 1
if [ -n "${KL_NATIVE_APP_RUN:-}" ]; then
    cp "$app" "$KL_NATIVE_APP_RUN/run" && chmod 755 "$KL_NATIVE_APP_RUN/run" || exit 1
    cd / || exit 1
    exec "$KL_NATIVE_APP_RUN/run"
fi

# Older launchers and the stock library start this entry without a shared RAM work directory.
case "$0" in
    /var/tmp/primitive-words.*/launch) work=${0%/launch} ;;
    *)
        work=$(mktemp -d /var/tmp/primitive-words.XXXXXX) || exit 1
        if ! cp "$0" "$work/launch"; then
            rmdir "$work"
            exit 1
        fi
        cd / || exit 1
        exec /bin/sh "$work/launch"
        ;;
esac
trap 'rm -rf "$work"' EXIT
cp "$app" "$work/run" && chmod 755 "$work/run" || exit 1
"$work/run"
