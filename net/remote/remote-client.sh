#!/usr/bin/env bash
set -e

cd "$(dirname "$0")"/../..

deployMethod="$1"
entrypointIp="$2"
clientToRun="$3"
if [[ -n $4 ]]; then
  export RUST_LOG="$4"
fi
benchTpsExtraArgs="$5"
clientIndex="$6"

missing() {
  echo "Error: $1 not specified"
  exit 1
}

[[ -n $deployMethod ]] || missing deployMethod
[[ -n $entrypointIp ]] || missing entrypointIp

source net/common.sh
loadConfigFile

threadCount=$(nproc)
if [[ $threadCount -gt 4 ]]; then
  threadCount=4
fi

case $deployMethod in
local|tar)
  PATH="$HOME"/.cargo/bin:"$PATH"
  export USE_INSTALL=1
  net/scripts/rsync-retry.sh -vPrc "$entrypointIp:~/.cargo/bin/*" ~/.cargo/bin/
  ;;
skip)
  ;;
*)
  echo "Unknown deployment method: $deployMethod"
  exit 1
esac

case $clientToRun in
safecoin-bench-tps)
  net/scripts/rsync-retry.sh -vPrc \
    "$entrypointIp":~/solana/config/bench-tps"$clientIndex".yml ./client-accounts.yml
  clientCommand="\
    safecoin-bench-tps \
      --entrypoint $entrypointIp:10015 \
      --faucet $entrypointIp:9900 \
      --duration 7500 \
      --sustained \
      --threads $threadCount \
      $benchTpsExtraArgs \
      --read-client-keys ./client-accounts.yml \
  "
  ;;
idle)
  # Add the faucet keypair to idle clients for convenience
  net/scripts/rsync-retry.sh -vPrc \
    "$entrypointIp":~/solana/config/faucet.json ~/solana/
  exit 0
  ;;
*)
  echo "Unknown client name: $clientToRun"
  exit 1
esac


cat > ~/solana/on-reboot <<EOF
#!/usr/bin/env bash
cd ~/solana

PATH="$HOME"/.cargo/bin:"$PATH"
export USE_INSTALL=1

echo "$(date) | $0 $*" >> client.log

(
  sudo SAFECOIN_METRICS_CONFIG="$SAFECOIN_METRICS_CONFIG" scripts/oom-monitor.sh
) > oom-monitor.log 2>&1 &
echo \$! > oom-monitor.pid
scripts/fd-monitor.sh > fd-monitor.log 2>&1 &
echo \$! > fd-monitor.pid
scripts/net-stats.sh  > net-stats.log 2>&1 &
echo \$! > net-stats.pid
! tmux list-sessions || tmux kill-session

tmux new -s "$clientToRun" -d "
  while true; do
    echo === Client start: \$(date) | tee -a client.log
    $metricsWriteDatapoint 'testnet-deploy client-begin=1'
    echo '$ $clientCommand' | tee -a client.log
    $clientCommand >> client.log 2>&1
    $metricsWriteDatapoint 'testnet-deploy client-complete=1'
  done
"
EOF
chmod +x ~/solana/on-reboot
echo "@reboot ~/solana/on-reboot" | crontab -

~/solana/on-reboot

sleep 1
tmux capture-pane -t "$clientToRun" -p -S -100
