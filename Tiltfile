docker_compose('./docker-compose.yml')

local_resource('cqrl-server', 'cargo build --release', serve_cmd=["./target/release/cqrl", "serve"], deps=['./src', './Cargo.toml', './Cargo.lock', './persistence', './events', './server', './errors', './parser'], trigger_mode=TRIGGER_MODE_AUTO)