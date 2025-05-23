[group('orbit')]
deploy_orbit_local:
  just deploy_orbit "local"

[private]
deploy_orbit network:
  #!/usr/bin/env bash
  set -ex
  echo "Deploying Orbit on {{network}} network"

  cd /tmp
  rm -rf /tmp/orbit
  git clone git@github.com:dfinity/orbit.git
  cd orbit/
  git checkout @orbit/station-v0.5.0
