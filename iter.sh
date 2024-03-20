#! /bin/bash
ignore_extensions=("lock", "yaml", "json", "sh", "toml", "parquet")
output_file="/tmp/exports/combined.md"

process_file() {
    local file=$1
    local filename="${file##*/}"
    local extension="${filename##*.}"
    for ext in "${ignore_extensions[@]}"; do 
        if [ "$ext" == "$extension" ]; then
            return
        fi
    done

    temp_file=$(mktemp) 
    cloc --strip-comments=nc "$file"


    # echo "**$filename**" >> "$output_file"
    # echo "" >> "$output_file"
    # echo "\`\`\`txt" >> "$output_file"
    # cat "$file" >> "$output_file"

    # echo "\`\`\`" >> "$output_file"
    # echo "" >> "$output_file"
    # echo "" >> "$output_file"
}

find_and_process() {
  local dir="$1"
  for file in "$dir"/*; do
    if [[ -f "$file" ]]; then
      process_file "$file"
    elif [[ -d "$file" ]]; then
      find_and_process "$file" 
    fi
  done
}

> "$output_file"  


find_and_process $1
