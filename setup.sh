#!/bin/bash

AUTHOR=$(git config user.name)
EMAIL=$(git config user.email)
read -rp "Enter the name of the project: " CRATENAME

FILES=(Cargo.toml README.md src/main.rs .github/workflows/ci.yml)

if ! command -v sd >/dev/null 2>&1; then
	echo "Please install 'sd' which can be installed with"
	echo "cargo install sd"
	exit 176
fi
if ! command -v fzf >/dev/null 2>&1; then
	echo "Cannot find fzf, please make sure it is installed and is locatedd in the path, addtionally see following link for install instructions"
	echo "https://github.com/junegunn/fzf#installation"
	exit 177
fi

read -rp "Please choose the project type (bin/lib): " TYPE
if [ "$TYPE" = "bin" ]; then
	sd "Cargo.lock" "#Cargo.lock" ".gitignore"
elif [ "$TYPE" = "lib" ]; then
	:
else
	echo "Not a valid option"
	exit 178
fi

read -rp "Please choose the Rust toolchain for the project (stable/nightly): " TOOLCHAIN
if [ "$TOOLCHAIN" = "stable" ] || [ "$TOOLCHAIN" = "nightly" ]; then
	echo "$TOOLCHAIN" > rust-toolchain
else
	echo "Not a valid Rust toolchain"
	exit 179
fi


LICENSE=$(curl -s "https://raw.githubusercontent.com/spdx/license-list-data/master/licenses.md" | grep -oP "^\[\K[\w-.]+" | sort | uniq | tr " " "\n" | fzf)
echo "Downloading $LICENSE License document from https://raw.githubusercontent.com/spdx/license-list/master/$LICENSE.txt"
curl -s "https://raw.githubusercontent.com/spdx/license-list/master/$LICENSE.txt" | sed "s//\n/g" > LICENSE || exit 188

echo "Replacing README"
printf '# CRATENAME\n\n## Usage\nAdd the following line to your Cargo.toml\n```toml\nCRATENAME = 0.1.0\n```' > README.md
echo "Replacing variables in ${FILES[@]}"
sd 'CRATENAME' "$CRATENAME" "${FILES[@]}"
sd 'AUTHOR' "$AUTHOR" "${FILES[@]}"
sd 'EMAIL' "$EMAIL" "${FILES[@]}"

echo "Self destructing..."
rm setup.sh
