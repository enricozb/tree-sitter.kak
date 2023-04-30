# ────────────── initialization ──────────────
declare-option str tree_sitter_config
declare-option str tree_sitter_socket

# directory to write buffer contents to.
declare-option -hidden str tree_sitter_dir

# timestamp to debounce buffer change hooks.
declare-option -hidden str tree_sitter_timestamp

# used for highlighting
declare-option -hidden range-specs tree_sitter_ranges
declare-option -hidden range-specs tree_sitter_ranges_spare

define-command tree-sitter-enable-buffer -docstring "start the tree-sitter server" %{
  evaluate-commands %sh{
    if [ -z "$kak_opt_tree_sitter_socket" ]; then
      if [ -n "$kak_opt_tree_sitter_config" ]; then
        config="--config $kak_opt_tree_sitter_config"
      fi

      printf "set-option global tree_sitter_socket '%s'\n" \
        $(kak-tree-sitter --daemonize --session $kak_session $config)
    fi
  }

  # 0. remove any extant hooks
  remove-hooks buffer tree-sitter

  # 1. send sync command to kak tree sitter to ask what buffer file to write to.
  tree-sitter-new-buffer

  # 2. setup hooks to write constantly to that file.
  hook -group tree-sitter buffer InsertIdle   .* tree-sitter-refresh
  hook -group tree-sitter buffer NormalIdle   .* tree-sitter-refresh
  hook -group tree-sitter buffer InsertChar   .* tree-sitter-refresh
  hook -group tree-sitter buffer InsertDelete .* tree-sitter-refresh
  hook -group tree-sitter buffer BufReload    .* tree-sitter-refresh

  hook -group tree-sitter buffer BufSetOption filetype=.* %{
    tree-sitter-set-language
    tree-sitter-refresh
  }

  # 3. add highlighter
  try %{
    add-highlighter buffer/ ranges tree_sitter_ranges
  }
}

define-command tree-sitter-refresh %{
  evaluate-commands -no-hooks %sh{
    if [ "$kak_timestamp" != "$kak_opt_tree_sitter_timestamp" ]; then
      echo 'write "%opt{tree_sitter_dir}/%val{timestamp}"'
      echo 'tree-sitter-parse-buffer'
      echo 'tree-sitter-highlight-buffer'
      echo 'set-option buffer tree_sitter_timestamp %val{timestamp}'
    fi
  }
}


# ────────────── tree-sitter requests ──────────────
define-command -hidden tree-sitter-sync-request -docstring "send sync request to tree-sitter" -params 1 %{
  evaluate-commands -no-hooks %sh{ echo "$1" | socat - UNIX-CONNECT:$kak_opt_tree_sitter_socket }
}

define-command -hidden tree-sitter-async-request -docstring "send async request to tree-sitter" -params 1 %{
  nop %sh{ { echo "$1" | socat - UNIX-CONNECT:$kak_opt_tree_sitter_socket; } > /dev/null 2>&1 < /dev/null & }
}

define-command tree-sitter-reload-config %{
  tree-sitter-async-request "
    type   = 'reload_config'
    config = '%opt{tree_sitter_config}'
  "
}

define-command tree-sitter-new-buffer %{
  tree-sitter-sync-request "
    type     = 'new_buffer'
    buffer   = '%val{bufname}'
    language = '%opt{filetype}'
  "
}

define-command tree-sitter-set-language %{
  tree-sitter-async-request "
    type     = 'set_language'
    buffer   = '%val{bufname}'
    language = '%opt{filetype}'
  "
}

define-command tree-sitter-parse-buffer %{
  tree-sitter-async-request "
    type      = 'parse_buffer'
    buffer    = '%val{bufname}'
    timestamp =  %val{timestamp}
  "
}

define-command tree-sitter-highlight-buffer %{
  tree-sitter-async-request "
    type      = 'highlight'
    buffer    = '%val{bufname}'
    timestamp =  %val{timestamp}
  "
}
