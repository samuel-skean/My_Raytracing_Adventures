# Credit to Jacob Cohen.
#!/bin/bash
diffWord() {
    # Using temp files to allow for redirection with <()
    local temp1="`mktemp`"
    local temp2="`mktemp`"

    cat "$1" > "$temp1"
    cat "$2" > "$temp2"

    printf "    Words added:   "
    git diff --no-index --word-diff=porcelain --unified=0 "$temp1" "$temp2"
    printf "    Words removed: "
    git diff --no-index --word-diff=porcelain --unified=0 "$temp1" "$temp2"

    rm -f "$temp1" "$temp2"
}

diffWord "$1" "$2"