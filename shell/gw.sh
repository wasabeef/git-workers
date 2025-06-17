#!/bin/bash
# Git Workers shell function with auto-cd
# Add this to your ~/.bashrc or ~/.zshrc:
# source /path/to/git-workers/shell/gw.sh

gw() {
    # Create temp file for SWITCH_TO
    local switch_file="/tmp/gw_switch_$$"
    
    # Run gw with environment variable
    GW_SWITCH_FILE="$switch_file" command gw "$@"
    local exit_code=$?
    
    # Check if switch file exists and cd if needed
    if [[ -f "$switch_file" ]]; then
        local new_dir=$(cat "$switch_file" 2>/dev/null)
        rm -f "$switch_file"
        if [[ -n "$new_dir" && -d "$new_dir" ]]; then
            cd "$new_dir"
        fi
    fi
    
    return $exit_code
}