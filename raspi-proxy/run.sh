set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

# SSH Config
REMOTE_LOCATION="10.42.0.1"
REMOTE_USERNAME="user"
REMOTE_PASSWORD="pass"

# Other constants
BINARY_NAME="raspi-proxy"
TARGET_HOST=${REMOTE_USERNAME}@${REMOTE_LOCATION}
TARGET_ARCH=aarch64-unknown-linux-gnu
SOURCE_PATH="./target/${TARGET_ARCH}/release/${BINARY_NAME}"
TARGET_PATH="/home/${REMOTE_USERNAME}/${BINARY_NAME}"

cargo build --release --target=$TARGET_ARCH $@ # Build
sshpass -p ${REMOTE_PASSWORD} scp ${SOURCE_PATH} ${TARGET_HOST}:${TARGET_PATH} # Upload
sshpass -p ${REMOTE_PASSWORD} ssh -t ${TARGET_HOST} "RUST_BACKTRACE=full RUST_LOG=info ${TARGET_PATH}" # Run