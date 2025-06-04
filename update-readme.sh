#!/bin/bash -e

cd "$(dirname "$0")" # Make sure to be in the directory where the script is located

# Install the cargo readme plugin if it is not yet installed
if ! cargo --list | grep -q readme; then cargo install cargo-readme; fi

# Generate README from the crate documentation, but remove the example
# configuration, because the include does not work with "cargo readme"
pushd "xrechnung" >/dev/null
cargo readme | sed '/^## Example Configuration.*/,$d' > README.md 
cp README.md ../README.md
popd >/dev/null


## Add the example configuration file to the README
cat << EOF >> README.md
You need to provide a TOML configuration file like this (also available at [xrechnung/examples/config.toml](xrechnung/examples/config.toml)):

\`\`\`toml
EOF

cat xrechnung/examples/config.toml >> README.md

cat << EOF >> README.md
\`\`\`
EOF


## Add documentation for the command line application to the README
cat << EOF >> README.md
## Command Line Application

This repository also contains a command line application for generating XRechnung files. It supports the following parameters:

\`\`\`
EOF

cargo run -- --help >> README.md

cat << EOF >> README.md
\`\`\`

The invoice hours CSV file (also available at [xrechnung_cmd/examples/invoice-lines.csv](xrechnung_cmd/examples/invoice-lines.csv)) could look like this then:

\`\`\`csv
EOF

cat xrechnung_cmd/examples/invoice-lines.csv >> README.md

cat << EOF >> README.md
\`\`\`

EOF
