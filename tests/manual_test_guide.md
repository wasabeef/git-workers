# Manual Testing Guide for Git Workers

## Testing Switch Command Fix

To test that the switch command properly updates the current worktree status:

1. **Setup Test Environment**

   ```bash
   # Create a test directory
   mkdir -p /tmp/gw-test
   cd /tmp/gw-test

   # Initialize bare repository
   git init --bare test-repo.bare
   cd test-repo.bare

   # Create initial commit
   git hash-object -w --stdin <<< "test" | xargs -I {} git update-index --add --cacheinfo 100644 {} README.md
   git write-tree | xargs -I {} git commit-tree {} -m "Initial commit" | xargs git update-ref refs/heads/main

   # Create worktrees
   git worktree add ../branch/main main
   git worktree add ../branch/feature-a -b feature-a
   git worktree add ../branch/feature-b -b feature-b
   ```

2. **Test Switch Functionality**

   ```bash
   # Source the shell function
   source /path/to/git-workers/shell/gw.sh

   # Run gw from bare repository
   cd /tmp/gw-test/test-repo.bare
   gw
   ```

3. **Expected Behavior**
   - Select "→ Switch worktree"
   - Choose a worktree (e.g., "feature-a")
   - The process should exit and your shell should change to that directory
   - Run `pwd` to confirm you're in the new directory
   - Run `gw` again and check that the selected worktree shows as "[current]"

## Testing Error Display Fix

1. **Test with No Worktrees**

   ```bash
   # Create empty repository
   mkdir -p /tmp/empty-test
   cd /tmp/empty-test
   git init

   # Run gw
   gw
   ```

2. **Expected Behavior**
   - Select operations that require worktrees (list, switch, search)
   - Should show "No worktrees found" message
   - After pressing any key, the menu should redraw cleanly without duplication

## Testing Search Function Fix

1. **With Existing Worktrees**
   - Select "? Search worktrees"
   - Enter a search term
   - If matches are found and you choose to switch, the process should exit
   - If you cancel or no matches found, should return to menu without issues

## Notes

- The shell function integration is critical for directory switching
- Make sure you're using the shell function (`gw`) not the binary directly
- The "SWITCH_TO:" marker is how the shell function knows to change directories
