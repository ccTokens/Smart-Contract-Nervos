#!/bin/bash

SOURCE=${BASH_SOURCE[0]}
while [ -L "$SOURCE" ]; do # resolve $SOURCE until the file is no longer a symlink
  DIR=$( cd -P "$( dirname "$SOURCE" )" >/dev/null 2>&1 && pwd )
  SOURCE=$(readlink "$SOURCE")
  [[ $SOURCE != /* ]] && SOURCE=$DIR/$SOURCE # if $SOURCE was a relative symlink, we need to resolve it relative to the path where the symlink file was located
done
BASEDIR=$( cd -P "$( dirname "$SOURCE" )" >/dev/null 2>&1 && pwd )

SCHEMA_PATH="${BASEDIR}/schemas"
DIST_RUST_PATH="${BASEDIR}/rust"
DIST_GO_PATH="${BASEDIR}/go"
DIST_JS_PATH="${BASEDIR}/js"
DIST_C_PATH="${BASEDIR}/c"

function compile() {
    local language=$1
    local schema_path=$2
    local dist_path=$3
    local suffix=".txt"

    case $1 in
    rust) suffix=".rs" ;;
    go)   suffix=".go" ;;
    js)   suffix=".js" ;;
    c)    suffix=".h" ;;
    esac

    if [[ ! -d $dist_path ]]; then
        mkdir -p $dist_path
    fi

    # walk through all directories recursively
    files=$(ls -a $schema_path)
    for file in $files; do
        if [[ $file != .* ]]; then
            if [ ! -d "${file}" ]; then
                echo "Compile ${schema_path}/${file} to ${dist_path}/${file%.*}${suffix}"
                case $language in
                js)
                    moleculec --language - --schema-file ${schema_path}/${file} --format json > ./tmp-schema.json
                    moleculec-es -inputFile ./tmp-schema.json -outputFile ${dist_path}/${file%.*}${suffix} -generateTypeScriptDefinition -hasBigInt
                    rm ./tmp-schema.json
                    ;;
                *)
                    moleculec --language $language --schema-file ${schema_path}/${file} >${dist_path}/${file%.*}${suffix}
                    ;;
                esac
            else
                compile $language $schema_path $dist_path
            fi
        fi
    done
}

case $1 in
rust)
    compile rust $SCHEMA_PATH $DIST_RUST_PATH/src/schemas
    cd $DIST_RUST_PATH
    cargo fmt --manifest-path="${BASEDIR}/Cargo.toml"
    ;;
go)
    compile go $SCHEMA_PATH $DIST_GO_PATH/src
    ;;
js)
    compile js $SCHEMA_PATH $DIST_JS_PATH/src
    ;;
c)
    compile c $SCHEMA_PATH $DIST_C_PATH/src
    ;;
*)
    echo "Unsupported compiling target."
    exit 0
    ;;
esac

echo "Done ✔"
