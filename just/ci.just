[group('ci')]
check_candid_is_up_to_date:
  just git_check_file_changed "candid/orchestrator.did"
  just git_check_file_changed "candid/user_canister.did"

[private]
git_check_file_changed file:
  git diff --exit-code --quiet HEAD -- "{{file}}"
