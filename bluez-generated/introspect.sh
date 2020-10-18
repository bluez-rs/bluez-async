#!/usr/bin/env bash
# Script to introspect bluez into specs/*.xml and generate src/*.rs from
# them.
#
# Introspection requires a running bluez daemon that is connected to devices
# with the features that you want to inspect. It also requires Bash >= 4 with
# associative array support. Bash >= 4 does not come with osx because of its
# GPLv3 license. Install it via homebrew.
# Set GDBUS='ssh pi@raspberrypi.local gdbus' to use remote gdbus.
# Set INTROSPECT=0 to skip introspection.
#
# Code generation requires dbus-codegen-rust from master.
# Run `cargo install --git=https://github.com/diwic/dbus-rs` to install.
# Set GENERATE=0 to skip code generation.

set -euo pipefail

cd "$(dirname "$0")"

GDBUS=${GDBUS:-gdbus}

if [ ${INTROSPECT:-1} = 1 ]; then
    $GDBUS introspect --system --dest org.bluez --object-path / --recurse \
        | grep -E '^ *(node|interface) .* {$' \
        | (
            declare -A interface_to_path

            while read keyword value _bracket; do
                if [ $keyword = 'node' ]; then
                    current_path=$value
                elif [ $keyword = 'interface' ]; then
                    interface_to_path[${value}]=$current_path
                else
                    echo "unexpected line $keyword $value $_bracket"
                    exit 1
                fi
            done

            for interface in ${!interface_to_path[@]}; do
                [[ $interface == org.bluez* ]] || continue
                echo $interface -- ${interface_to_path[${interface}]}
                $GDBUS introspect \
                    --system \
                    --dest=org.bluez \
                    --object-path=${interface_to_path[${interface}]} \
                    --xml \
                    | xmllint --format - \
                    | grep -v '^ *<node name=".*"/>$' \
                        > specs/$interface.xml
            done
        )
fi

if [ ${GENERATE:-1} = 1 ]; then
    echo "// Generated by introspect.sh" > src/lib.rs
    for file in specs/org.bluez.*.xml; do
        interface=$(
            echo $file \
                | sed -e 's:^specs/::' -e 's:[.]xml$::'
        )
        modname=$(
            echo $interface \
                | sed -e 's/^org.bluez.//' \
                | tr '[:upper:]' '[:lower:]'
        )
        dbus-codegen-rust \
            --file=$file \
            --interfaces=$interface \
            --client=nonblock \
            --methodtype=none \
            > src/$modname.rs
        echo "pub mod $modname;" >> src/lib.rs
        echo "pub use $modname::*;" >> src/lib.rs
    done
    cargo fmt
fi
